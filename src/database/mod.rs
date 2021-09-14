//! Kairoi's database, responsible for storing jobs and rules, and triggering jobs execution.
//!
//! The Database is one of the 3 mains components of the Kairoi architecture. It uses channels to
//! receive queries from the Controller, to send query responses to the Controller, to send
//! execution requests to the Processor, and to receive execution responses from the Processor. The
//! database runs in its own process, at its own framerate. The job and rule storage is delegated
//! to a [`Storage`]. The [`Storage`] is initialized at the start of the database process.

mod execution;
mod framerate;
mod query;
mod storage;

use chrono::DateTime;
use chrono::offset::Utc;
use crate::query::{Request as QueryRequest, Response as QueryResponse};
use log::debug;
use self::execution::{Client as ExecutionClient, Job as ExecutionJob, Receiver as ExecutionReceiver, Runner as ExecutionRunner, Sender as ExecutionSender};
use self::framerate::Clock;
use self::query::Handler as QueryHandler;
use self::storage::{Job, JobStatus, Runner, Storage};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub struct Database {
    storage: Storage,
    execution_client: ExecutionClient,
    query_handler: QueryHandler,
    current_datetime: DateTime<Utc>,
}

impl Database {
    /// Start the Database, spawning a thread and returning the join handle.
    pub fn start(
        query_link: (Sender<QueryResponse>, Receiver<QueryRequest>),
        execution_link: (ExecutionSender, ExecutionReceiver),
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut database = Database {
                storage: Storage::new(),
                execution_client: ExecutionClient::new(execution_link),
                query_handler: QueryHandler::new(query_link),
                current_datetime: Utc::now(),
            };

            match database.storage.initialize() {
                Ok(_triggered_jobs) => {
                    // @TODO: Re-process "triggered" jobs when booting up (causing duplicated job
                    // executions), because there is no way to know results of previous executions.
                },
                Err(_) => {
                    panic!("Unable to initialize the storage from data persisted to the file system.");
                },
            };

            let clock = Clock::with_framerate(128);
            clock.start(|| {
                database.current_datetime = Utc::now();

                database.query_handler.handle(&database.current_datetime, &mut database.storage);

                database.trigger_execution();
                database.handle_responses();
            });
        })
    }

    /// Check every waiting job, and trigger the execution when needed.
    fn trigger_execution(&mut self) {
        let jobs = self.storage.get_jobs_to_execute(&self.current_datetime);
        let mut triggering = Vec::with_capacity(jobs.len());
        let mut failing: Vec<Job> = Vec::with_capacity(jobs.len());
        for job in jobs {
            // Find a matching Runner.
            match self.storage.pair(job.get_identifier()) {
                Some(rule) => {
                    triggering.push((
                        job.clone(),
                        rule.clone(),
                    ));
                },
                None => {
                    failing.push(job.clone());
                },
            };
        };

        // Trigger all jobs that have been paired with a runner.
        for (job, rule) in &triggering {
            debug!("TRIGGER {:?} with {:?} at {}.", &job, &rule, &self.current_datetime);
            let runner = rule.get_runner().clone();
            let modified = Job::new(
                job.get_identifier().clone(),
                job.get_execution().clone(),
                JobStatus::Triggered,
            );
            if let Err(_) = self.storage.set_job(modified) {
                continue;
            };

            self.execution_client.trigger(
                ExecutionJob { identifier: job.get_identifier().clone() },
                ExecutionRunner::from(runner),
            );
        };

        // Mark all jobs that haven't as failed.
        for job in &failing {
            debug!("Unable to find a Rule pairing {:?}.", &job);
            debug!("MARK AS FAILED {:?} at {}.", &job, &self.current_datetime);
            let job = Job::new(
                job.get_identifier().clone(),
                job.get_execution().clone(),
                JobStatus::Failed,
            );
            if let Err(_) = self.storage.set_job(job) {
                continue;
            }
        };
    }

    /// Pull all received execution responses and handle them.
    fn handle_responses(&mut self) {
        let responses = self.execution_client.pull_responses();

        for response in responses {
            match self.storage.get_job(&response.job) {
                Some(job) => {
                    let job = match response.result {
                        Ok(_) => {
                            debug!("MARK AS EXECUTED {:?} at {}.", job, &self.current_datetime);

                            Job::new(
                                job.get_identifier().clone(),
                                job.get_execution().clone(),
                                JobStatus::Executed,
                            )
                        },
                        Err(_) => {
                            debug!("MARK AS FAILED {:?} at {}.", job, &self.current_datetime);

                            Job::new(
                                job.get_identifier().clone(),
                                job.get_execution().clone(),
                                JobStatus::Failed,
                            )
                        },
                    };
                    // If there is a storage error, act like there was no response from the
                    // processor (leave the job in TRIGGERED state).
                    if let Err(_) = self.storage.set_job(job) {
                        continue;
                    };
                },
                None => {},
            };
        }
    }
}

/// Convert Storage Runner into ExecutionRunner.
impl From<Runner> for ExecutionRunner {
    fn from(runner: Runner) -> Self {
        match runner {
            Runner::Amqp { dsn, exchange, routing_key } => Self::Amqp { dsn, exchange, routing_key },
            Runner::Shell { command } => Self::Shell { command },
        }
    }
}
