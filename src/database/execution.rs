use chrono::DateTime;
use chrono::offset::Utc;
use crate::database::storage::{Job, JobStatus, Runner, Storage};
use crate::execution::{Request, Response};
use crate::execution::job::Job as ExecutionJob;
use crate::execution::runner::Runner as ExecutionRunner;
use log::debug;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use uuid::Uuid;

pub struct Handler {
    execution_link: (Sender<Request>, Receiver<Response>),
    triggered: HashMap<Uuid, Request>,
}

impl Handler {
    /// Create a new handler on the given execution link..
    pub fn new(execution_link: (Sender<Request>, Receiver<Response>)) -> Handler {
        Handler {
            execution_link: execution_link,
            triggered: HashMap::new(),
        }
    }

    /// Handle the execution link.
    pub fn handle(&mut self, current_datetime: &DateTime<Utc>, storage: &mut Storage) {
        self.trigger(current_datetime, storage);
        self.receive_responses(current_datetime, storage);
    }

    /// Check every waiting job, to notify when it should be executed.
    fn trigger(&mut self, current_datetime: &DateTime<Utc>, storage: &mut Storage) {
        let jobs = storage.get_jobs_to_execute(current_datetime);
        let mut triggering = Vec::with_capacity(jobs.len());
        let mut failing = Vec::with_capacity(jobs.len());
        for job in jobs {
            // Find a matching Runner.
            match storage.pair(job.get_identifier()) {
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
            debug!("TRIGGER {:?} with {:?} at {}.", &job, &rule, current_datetime);
            let runner = rule.get_runner().clone();
            let modified = Job::new(
                job.get_identifier().clone(),
                job.get_execution().clone(),
                JobStatus::Triggered,
            );
            if let Err(_) = storage.set_job(modified) {
                continue;
            };
            let identifier = Uuid::new_v4();
            let request = Request::new(
                identifier,
                ExecutionJob::new(job.get_identifier().clone()),
                ExecutionRunner::from(runner),
            );
            self.triggered.insert(identifier, request.clone());
            if let Err(_) = self.execution_link.0.send(request) {
                panic!("Execution channel disconnected.");
            };
        };

        // Mark all jobs that haven't as failed.
        for job in &failing {
            debug!("Unable to find a Rule pairing {:?}.", &job);
            debug!("MARK AS FAILED {:?} at {}.", &job, current_datetime);
            let job = Job::new(
                job.get_identifier().clone(),
                job.get_execution().clone(),
                JobStatus::Failed,
            );
            if let Err(_) = storage.set_job(job) {
                continue;
            }
        };
    }

    /// Pull all received notification confirmations and handle them.
    fn receive_responses(&mut self, current_datetime: &DateTime<Utc>, storage: &mut Storage) {
        let mut responses = Vec::new();

        loop {
            match self.execution_link.1.try_recv() {
                Ok(response) => {
                    responses.push(response);
                },
                Err(error) => match error {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => panic!("Execution channel disconnected."),
                },
            }
        };

        for response in responses {
            match self.triggered.remove(response.get_identifier()) {
                Some(request) => {
                    let job = request.get_job();
                    match storage.get_job(job.get_identifier()) {
                        Some(job) => {
                            let job = match response.get_result() {
                                Ok(_) => {
                                    debug!("MARK AS EXECUTED {:?} at {}.", job, current_datetime);

                                    Job::new(
                                        job.get_identifier().clone(),
                                        job.get_execution().clone(),
                                        JobStatus::Executed,
                                    )
                                },
                                Err(_) => {
                                    debug!("MARK AS FAILED {:?} at {}.", job, current_datetime);

                                    Job::new(
                                        job.get_identifier().clone(),
                                        job.get_execution().clone(),
                                        JobStatus::Failed,
                                    )
                                },
                            };
                            // If there is a storage error, act like there was no response from the
                            // processor (leave the job in TRIGGERED state).
                            if let Err(_) = storage.set_job(job) {
                                continue;
                            };
                        },
                        None => {},
                    };
                },
                None => {},
            };
        };
    }
}

/// Convert Runner into ExecutionRunner.
impl From<Runner> for ExecutionRunner {
    fn from(runner: Runner) -> Self {
        match runner {
            Runner::Amqp { dsn, exchange, routing_key } => Self::Amqp { dsn, exchange, routing_key },
            Runner::Shell { command } => Self::Shell { command },
        }
    }
}
