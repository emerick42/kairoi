//! Kairoi's processor, responsible for executing jobs.
//!
//! The processor is one of the 3 main components of the Kairoi architecture. It uses channels to
//! receive execution requests from the database, and send execution responses in return. The
//! processor runs in its own process. Since it only reacts on externel calls (requests from the
//! database, responses from runners, etc.), it uses a "select/epoll" model.

#[cfg(feature = "runner-amqp")]
mod amqp;
pub mod protocol;
#[cfg(feature = "runner-shell")]
mod shell;

use crossbeam_channel::Receiver as CrossbeamReceiver;
use crossbeam_channel::Select;
use crossbeam_channel::Sender as CrossbeamSender;
#[allow(unused_imports)]
use crossbeam_channel::unbounded;
use log::debug;
#[cfg(feature = "runner-amqp")]
use self::amqp::{Amqp, Receiver as AmqpReceiver, Request as AmqpRequest, Sender as AmqpSender};
use self::protocol::Request as ProtocolRequest;
use self::protocol::Response as ProtocolResponse;
#[allow(unused_imports)]
use self::protocol::Runner as ProtocolRunner;
#[cfg(feature = "runner-shell")]
use self::shell::{Receiver as ShellReceiver, Request as ShellRequest, Sender as ShellSender, Shell};
use std::thread;

pub type Sender = CrossbeamSender<ProtocolResponse>;
pub type Receiver = CrossbeamReceiver<ProtocolRequest>;

pub struct Processor {}

impl Processor {
    /// Start the processor, spawning a thread and returning the join handle.
    pub fn start((sender, receiver): (Sender, Receiver)) -> thread::JoinHandle<()> {
        thread::Builder::new().name("kairoi/proc".to_string()).spawn(move || {
            let mut dispatcher = Dispatcher::new(sender, receiver);

            dispatcher.run()
        }).unwrap()
    }
}

/// Defer execution to runners selected in the execution request.
pub struct Dispatcher {
    #[cfg(feature = "runner-amqp")]
    amqp: Amqp,
    main_link: (Sender, Receiver),
    #[cfg(feature = "runner-shell")]
    shell_link: (ShellSender, ShellReceiver),
    #[cfg(feature = "runner-amqp")]
    amqp_link: (AmqpSender, AmqpReceiver),
}

impl Dispatcher {
    /// Create a new runner.
    pub fn new(sender: Sender, receiver: Receiver) -> Self {
        Self {
            #[cfg(feature = "runner-amqp")]
            amqp: Amqp::new(),
            main_link: (sender, receiver),
            #[cfg(feature = "runner-shell")]
            shell_link: unbounded(),
            #[cfg(feature = "runner-amqp")]
            amqp_link: unbounded(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut select = Select::new();
            let main_operation = select.recv(&self.main_link.1);
            #[cfg(feature = "runner-shell")]
            let shell_operation = select.recv(&self.shell_link.1);
            #[cfg(feature = "runner-amqp")]
            let amqp_operation = select.recv(&self.amqp_link.1);

            let operation = select.select();
            match operation.index() {
                // Listening to execution requests from the database on the main receiver.
                index if index == main_operation => {
                    match operation.recv(&self.main_link.1) {
                        Ok(request) => {
                            self.handle_request(&request);
                        },
                        Err(_) => {
                            panic!("The Execution Request channel between the database and the processor has been disconnected.");
                        },
                    };
                },
                // Listening to execution responses from Shell runners on the shell receiver.
                #[cfg(feature = "runner-shell")]
                index if index == shell_operation => {
                    match operation.recv(&self.shell_link.1) {
                        Ok(shell_response) => {
                            let response = ProtocolResponse {
                                identifier: shell_response.identifier,
                                result: shell_response.result,
                            };
                            if let Err(_) = self.main_link.0.send(response) {
                                panic!("Execution channel disconnected.");
                            };
                        },
                        Err(_) => {
                            panic!("The Execution Response channel between the shell and the processor has been disconnected.");
                        }
                    };
                },
                // Listening to execution responses from Amqp runners on the amqp receiver.
                #[cfg(feature = "runner-amqp")]
                index if index == amqp_operation => {
                    match operation.recv(&self.amqp_link.1) {
                        Ok(shell_response) => {
                            let response = ProtocolResponse {
                                identifier: shell_response.identifier,
                                result: shell_response.result,
                            };
                            if let Err(_) = self.main_link.0.send(response) {
                                panic!("Execution channel disconnected.");
                            };
                        },
                        Err(_) => {
                            panic!("The Execution Response channel between the shell and the processor has been disconnected.");
                        }
                    };
                },
                _ => unreachable!(),
            }
        }
    }

    /// Handle the incoming execution request.
    fn handle_request(&mut self, request: &ProtocolRequest) {
        match self.execute(request) {
            Ok(_) => {},
            Err(_) => {
                debug!("Unable to execute {:?}, unsupported runner.", request);
                // Mark the job as failed.
                let response = ProtocolResponse {
                    identifier: request.identifier,
                    result: Err(()),
                };
                if let Err(_) = self.main_link.0.send(response) {
                    panic!("Execution channel disconnected.");
                };
            },
        };
    }

    /// Execute the given request, using the given producer to send the execution response
    /// asynchronously to the calling processor. When no runner can be matched, it directly returns
    /// an error, never sending a result on the according channel.
    fn execute(&mut self, request: &ProtocolRequest) -> Result<(), ()> {
        #[allow(unreachable_patterns)]
        match &request.runner {
            #[cfg(feature = "runner-shell")]
            ProtocolRunner::Shell { command } => {
                Shell::execute(
                    ShellRequest::new(request.identifier, request.job_identifier.clone(), command.clone()),
                    &self.shell_link.0,
                )
            },
            #[cfg(feature = "runner-amqp")]
            ProtocolRunner::Amqp { dsn, exchange, routing_key } => {
                self.amqp.execute(
                    AmqpRequest::new(
                        request.identifier,
                        request.job_identifier.clone(),
                        dsn.clone(),
                        exchange.clone(),
                        routing_key.clone(),
                    ),
                    &self.amqp_link.0,
                )
            },
            _ => {
                Err(())
            },
        }
    }
}
