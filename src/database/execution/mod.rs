//! Database job execution, handling execution requests with the processor.
//!
//! It provides a [`Client`] to transmit job execution requests to the processor using the
//! [`Client::trigger`] method. Responses should be pulled regularly using
//! [`Client::pull_responses`]. The client uses standard [`std::sync::mpsc::Sender`] and
//! [`std::sync::mpsc::Receiver`] as the underlying link with the processor.

pub mod protocol;

use crossbeam_channel::{Receiver as CrossbeamReceiver, Sender as CrossbeamSender, TryRecvError};
use self::protocol::{Request as ProtocolRequest, Response as ProtocolResponse, Runner as ProtocolRunner};
use std::collections::HashMap;
use std::result::Result as StdResult;
use uuid::Uuid;

pub type Runner = ProtocolRunner;
pub struct Result {
    pub job: String,
    pub result: StdResult<(), ()>,
}

pub type Sender = CrossbeamSender<ProtocolRequest>;
pub type Receiver = CrossbeamReceiver<ProtocolResponse>;

pub struct Client {
    execution_link: (Sender, Receiver),
    triggered: HashMap<Uuid, ProtocolRequest>,
}

impl Client {
    /// Create a new client, using the given execution link to trigger job execution.
    pub fn new(execution_link: (Sender, Receiver)) -> Self {
        Self {
            execution_link: execution_link,
            triggered: HashMap::new(),
        }
    }

    /// Trigger the execution for the job having the given identifier, on the given runner.
    pub fn trigger(&mut self, job: String, runner: Runner) -> () {
        let identifier = Uuid::new_v4();
        let request = ProtocolRequest {
            identifier: identifier,
            job_identifier: job,
            runner: runner,
        };

        self.triggered.insert(identifier, request.clone());

        if let Err(_) = self.execution_link.0.send(request) {
            panic!("Execution channel disconnected.");
        };
    }

    /// Pull all execution results received through the execution link.
    pub fn pull_results(&mut self) -> Vec<Result> {
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

        responses.into_iter().filter_map(|response: ProtocolResponse| {
            match self.triggered.remove(&response.identifier) {
                Some(request) => Some(Result {
                    job: request.job_identifier,
                    result: response.result,
                }),
                None => None,
            }
        }).collect()
    }
}
