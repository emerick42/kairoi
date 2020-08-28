use crate::database::execution_context::{ExecutionContext, Job, JobStatus};
use crate::execution::{Request, Response};
use crate::execution::job::Job as ExecutionJob;
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
    pub fn handle(&mut self, context: &mut ExecutionContext) {
        self.trigger(context);
        self.receive_responses(context);
    }

    /// Check every waiting job, to notify when it should be executed.
    fn trigger(&mut self, context: &mut ExecutionContext) {
        let current_datetime = context.get_current_datetime().clone();

        let jobs = context.get_jobs_to_execute();
        let mut triggering = Vec::with_capacity(jobs.len());
        let mut failing = Vec::with_capacity(jobs.len());
        for job in jobs {
            // Find a matching Runner.
            match context.pair(job.get_identifier()) {
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
        for (job, rule) in triggering {
            debug!("TRIGGER {:?} with {:?} at {}.", &job, &rule, current_datetime);
            let runner = rule.get_runner().clone();
            let modified = Job::new(
                job.get_identifier().clone(),
                job.get_execution().clone(),
                JobStatus::Triggered,
            );
            context.set_job(modified);
            let identifier = Uuid::new_v4();
            let request = Request::new(
                identifier,
                ExecutionJob::new(job.get_identifier().clone()),
                runner,
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
            context.set_job(job);
        };
    }

    /// Pull all received notification confirmations and handle them.
    fn receive_responses(&mut self, context: &mut ExecutionContext) {
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
                    let current_datetime = context.get_current_datetime();
                    match context.get_job(job.get_identifier()) {
                        Some(job) => {
                            match response.get_result() {
                                Ok(_) => {
                                    debug!("MARK AS EXECUTED {:?} at {}.", job, current_datetime);
                                    let job = Job::new(
                                        job.get_identifier().clone(),
                                        job.get_execution().clone(),
                                        JobStatus::Executed,
                                    );
                                    context.set_job(job);
                                },
                                Err(_) => {
                                    debug!("MARK AS FAILED {:?} at {}.", job, current_datetime);
                                    let job = Job::new(
                                        job.get_identifier().clone(),
                                        job.get_execution().clone(),
                                        JobStatus::Failed,
                                    );
                                    context.set_job(job);
                                },
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
