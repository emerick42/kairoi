use crate::database::ExecutionContext;
use crate::database::job::Status;
use crate::execution::{Request, Response};
use crate::execution::job::Job;
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
        let current_datetime = context.get_current_datetime();
        // Retrieve all jobs ready to be executed.
        let mut jobs = Vec::new();
        for job in context.get_jobs() {
            match job.get_status() {
                Status::Planned if *job.get_execution() <= current_datetime => {
                    jobs.push(job.clone());
                },
                _ => {},
            };
        };
        // Trigger the execution for each job.
        for job in &mut jobs {
            // Find a matching Runner.
            match context.pair(&job.get_identifier()) {
                Some(rule) => {
                    debug!("TRIGGER {:?} with {:?} at {}.", job, rule, current_datetime);
                    let runner = rule.get_runner().clone();
                    job.set_triggered().unwrap();
                    context.set_job(job.clone());
                    let identifier = Uuid::new_v4();
                    let request = Request::new(
                        identifier,
                        Job::new(job.get_identifier().clone()),
                        runner,
                    );
                    self.triggered.insert(identifier, request.clone());
                    if let Err(_) = self.execution_link.0.send(request) {
                        panic!("Execution channel disconnected.");
                    };
                },
                None => {
                    debug!("Unable to find a Rule pairing {:?}.", job);
                    debug!("MARK AS FAILED {:?} at {}.", job, current_datetime);
                    let mut job = job.clone();
                    job.set_failed().unwrap();
                    context.set_job(job.clone());
                },
            };
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
                    match context.get_job_mut(job.get_identifier()) {
                        Some(job) => {
                            match response.get_result() {
                                Ok(_) => {
                                    debug!("MARK AS EXECUTED {:?} at {}.", job, current_datetime);
                                    job.set_executed().unwrap();
                                },
                                Err(_) => {
                                    debug!("MARK AS FAILED {:?} at {}.", job, current_datetime);
                                    job.set_failed().unwrap();
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
