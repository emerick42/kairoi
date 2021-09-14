//! Database job execution, handling execution requests with the processor.
//!
//! It provides a [`Client`] to transmit job execution requests to the processor using the
//! [`Client::trigger`] method. Responses should be pulled regularly using
//! [`Client::pull_responses`]. The client uses standard [`std::sync::mpsc::Sender`] and
//! [`std::sync::mpsc::Receiver`] as the underlying link with the processor.

use crate::execution::{Request as ExecutionRequest, Response as ExecutionResponse};
use crate::execution::job::Job as ExecutionJob;
use crate::execution::runner::Runner as ExecutionRunner;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver as MpscReceiver, Sender as MpscSender, TryRecvError};
use uuid::Uuid;

pub type Runner = ExecutionRunner;
pub struct Job {
    pub identifier: String,
}
pub struct Response {
    pub job: String,
    pub result: Result<(), ()>,
}

pub type Sender = MpscSender<ExecutionRequest>;
pub type Receiver = MpscReceiver<ExecutionResponse>;

pub struct Client {
    execution_link: (MpscSender<ExecutionRequest>, MpscReceiver<ExecutionResponse>),
    triggered: HashMap<Uuid, ExecutionRequest>,
}

impl Client {
    /// Create a new client, using the given execution link to trigger job execution.
    pub fn new(execution_link: (MpscSender<ExecutionRequest>, MpscReceiver<ExecutionResponse>)) -> Self {
        Self {
            execution_link: execution_link,
            triggered: HashMap::new(),
        }
    }

    /// Trigger the execution for the job having the given identifier, on the given runner.
    pub fn trigger(&mut self, job: Job, runner: Runner) -> () {
        let identifier = Uuid::new_v4();
        let request = ExecutionRequest::new(
            identifier,
            ExecutionJob::new(job.identifier),
            ExecutionRunner::from(runner),
        );

        self.triggered.insert(identifier, request.clone());

        if let Err(_) = self.execution_link.0.send(request) {
            panic!("Execution channel disconnected.");
        };
    }

    /// Pull all responses received through the execution link.
    pub fn pull_responses(&mut self) -> Vec<Response> {
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

        responses.into_iter().filter_map(|response: ExecutionResponse| {
            match self.triggered.remove(response.get_identifier()) {
                Some(request) => Some(Response {
                    job: request.get_job().get_identifier().to_string(),
                    result: *response.get_result(),
                }),
                None => None,
            }
        }).collect()
    }
}
