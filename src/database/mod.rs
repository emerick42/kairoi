//! Kairoi's database, responsible for storing jobs and rules, and triggering jobs execution.
//!
//! The Database is one of the 3 mains components of the Kairoi architecture. It uses channels to
//! receive queries from the Controller, to send query responses to the Controller, to send
//! execution requests to the Processor, and to receive execution responses from the Processor. The
//! database runs in its own process, at its own framerate. The job and rule storage is delegated
//! to a [`Storage`]. The [`Storage`] is initialized at the start of the database process.

mod framerate;
mod query;
mod storage;
pub mod execution;

use chrono::DateTime;
use chrono::offset::Utc;
use crate::query::{Request as QueryRequest, Response as QueryResponse};
use log::debug;
use self::execution::Client as ExecutionClient;
use self::execution::Receiver as UnderlyingExecutionReceiver;
use self::execution::Result as ExecutionResult;
use self::execution::Runner as ExecutionRunner;
use self::execution::Sender as UnderlyingExecutionSender;
use self::framerate::Clock;
use self::query::Handler as QueryHandler;
use self::storage::{Job, JobStatus, Runner, Storage};
use self::storage::Configuration as StorageConfiguration;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub struct Database {
    storage: Storage,
    execution_client: ExecutionClient,
    query_handler: QueryHandler,
    current_datetime: DateTime<Utc>,
    unhandeld_results: Vec<ExecutionResult>,
}

pub type ExecutionSender = UnderlyingExecutionSender;
pub type ExecutionReceiver = UnderlyingExecutionReceiver;
pub struct Configuration {
    pub persistence: bool,
    pub storage_persistence_fsync_on_persist: bool,
    pub framerate: u16,
}

impl Database {
    /// Start the Database, spawning a thread and returning the join handle.
    pub fn start(
        query_link: (Sender<QueryResponse>, Receiver<QueryRequest>),
        execution_link: (ExecutionSender, ExecutionReceiver),
        configuration: Configuration,
    ) -> thread::JoinHandle<()> {
        thread::Builder::new().name("kairoi/db".to_string()).spawn(move || {
            let mut database = Database {
                storage: Storage::new(StorageConfiguration {
                    persistence: configuration.persistence,
                    persistence_fsync_on_persist: configuration.storage_persistence_fsync_on_persist,
                }),
                execution_client: ExecutionClient::new(execution_link),
                query_handler: QueryHandler::new(query_link),
                current_datetime: Utc::now(),
                unhandeld_results: Vec::new(),
            };

            match database.storage.initialize() {
                Ok(triggered_jobs) => {
                    // Re-process "triggered" jobs when booting up (causing duplicated job
                    // executions), because there is no way to know results of previous executions.
                    database.trigger_execution(triggered_jobs);
                },
                Err(_) => {
                    panic!("Unable to initialize the storage from data persisted to the file system.");
                },
            };

            let clock = Clock::with_framerate(configuration.framerate);
            clock.start(|| {
                database.current_datetime = Utc::now();

                database.query_handler.handle(&database.current_datetime, &mut database.storage);

                let jobs = database.storage.get_jobs_to_execute(&database.current_datetime);
                database.trigger_execution(jobs);
                database.handle_results();
            });
        }).unwrap()
    }

    /// Check every waiting job, and trigger the execution when needed.
    fn trigger_execution(&mut self, jobs: Vec<Job>) {
        let mut triggering = Vec::with_capacity(jobs.len());
        let mut failing = Vec::with_capacity(jobs.len());
        for job in &jobs {
            // Find a matching Runner.
            match self.storage.pair(job.get_identifier()) {
                Some(rule) => {
                    triggering.push((
                        job,
                        rule,
                    ));
                },
                None => {
                    failing.push(job);
                },
            };
        };

        // Trigger all jobs that have been paired with a runner.
        for (job, rule) in &triggering {
            debug!("TRIGGER {:?} with {:?} at {}.", job, &rule, &self.current_datetime);
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
                job.get_identifier().clone(),
                ExecutionRunner::from(runner),
            );
        };

        // Mark all jobs that haven't as failed.
        for job in &failing {
            debug!("Unable to find a Rule pairing {:?}.", job);
            debug!("MARK AS FAILED {:?} at {}.", job, &self.current_datetime);
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

    /// Pull all received execution results and handle them.
    ///
    /// It uses an in-memory storage to remember previously pulled results, allowing new job states
    /// to be written later in case of temporary persistent storage error. Jobs are still
    /// considered in the `triggered` state as long as the results is not properly written in the
    /// persistent storage. It must be noted that this storage will grow indefinitely if the
    /// persistent storage is unwritable for a long time.
    fn handle_results(&mut self) {
        let mut results: Vec<_> = self.unhandeld_results.drain(..).collect();
        results.extend(self.execution_client.pull_results());

        results.retain(|response| {
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

                    match self.storage.set_job(job) {
                        Ok(_) => false,
                        Err(_) => true,
                    }
                },
                None => false,
            }
        });

        self.unhandeld_results = results;
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
