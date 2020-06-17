use crate::execution::{Request, Response};
use crate::execution::runner::Runner as ExecutionRunner;
use crate::processor::runner::Runner;
use log::debug;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

/// A runner executing a shell script with the job identifier as parameter.
pub struct Shell {}

impl Shell {
    /// Create a new shell runner.
    pub fn new() -> Shell {
        Shell {}
    }
}

impl Runner for Shell {
    /// Execute the given job if the runner configuration is of type shell.
    fn execute(&self, request: &Request, producer: &Sender<Response>) -> Result<(), ()> {
        match request.get_runner() {
            ExecutionRunner::Shell { command: command_line } => {
                debug!("Executing {:?}.", request);
                let command_line = command_line.clone();
                let job = request.get_job().get_identifier().to_string();
                let request = *request.get_identifier();
                let producer = producer.clone();

                thread::spawn(move || {
                    let mut command = Command::new("sh");

                    command
                        .arg(command_line)
                        .arg(job)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                    ;
                    match command.status() {
                        Ok(exit_status) => {
                            debug!("Shell runner exiting with status '{:?}'.", exit_status);
                            producer.send(Response::new(request, Ok(()))).unwrap();
                        },
                        Err(error) => {
                            debug!("Shell runner failed to execute (error: '{:?}').", error);
                            producer.send(Response::new(request, Err(()))).unwrap();
                        },
                    };
                });

                Ok(())
            },
        }
    }
}
