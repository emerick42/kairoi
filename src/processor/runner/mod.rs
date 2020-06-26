#[cfg(feature = "runner-amqp")]
mod amqp;
#[cfg(feature = "runner-shell")]
mod shell;

#[cfg(feature = "runner-amqp")]
use amqp::{Amqp, Request as AmqpRequest};
use crate::execution::{Request, Response};
use crate::execution::runner::Runner as ExecutionRunner;
#[cfg(feature = "runner-shell")]
use shell::{Request as ShellRequest, Shell};
use std::sync::mpsc::Sender;

/// Defer execution to runners selected in the execution request.
pub struct Runner {
    amqp: Amqp,
}

impl Runner {
    /// Create a new runner.
    pub fn new() -> Runner {
        Runner {
            amqp: Amqp::new(),
        }
    }

    /// Execute the given request, using the given producer to send the execution response
    /// asynchronously to the calling processor. Do not send an execution response when resulting
    /// an error.
    pub fn execute(&mut self, request: &Request, producer: &Sender<Response>) -> Result<(), ()> {
        #[allow(unreachable_patterns)]
        match request.get_runner() {
            #[cfg(feature = "runner-shell")]
            ExecutionRunner::Shell { command } => {
                Shell::execute(
                    ShellRequest::new(*request.get_identifier(), request.get_job().clone(), command.clone()),
                    producer,
                )
            },
            #[cfg(feature = "runner-amqp")]
            ExecutionRunner::Amqp { dsn, exchange, routing_key } => {
                self.amqp.execute(
                    AmqpRequest::new(
                        *request.get_identifier(),
                        request.get_job().clone(),
                        dsn.clone(),
                        exchange.clone(),
                        routing_key.clone(),
                    ),
                    producer,
                )
            },
            _ => {
                Err(())
            },
        }
    }
}
