use crate::execution::Response;
use log::debug;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use uuid::Uuid;
use crate::execution::job::Job;

/// An execution request about a job paired with a shell runner.
#[derive(Debug)]
pub struct Request {
    identifier: Uuid,
    job: Job,
    command: String,
}

impl Request {
    /// Create a new shell request.
    pub fn new(identifier: Uuid, job: Job, command: String) -> Request {
        Request {
            identifier: identifier,
            job: job,
            command: command,
        }
    }
}

/// A runner executing a shell script with the job identifier as parameter.
pub struct Shell {}

impl Shell {
    /// Execute the given job if the runner configuration is of type shell.
    pub fn execute(request: Request, producer: &Sender<Response>) -> Result<(), ()> {
        debug!("Executing {:?}.", request);
        let producer = producer.clone();

        thread::spawn(move || {
            let mut command = Command::new("sh");

            command
                .arg(request.command)
                .arg(request.job.get_identifier())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
            ;
            match command.status() {
                Ok(exit_status) => {
                    debug!("Shell runner exiting with status '{:?}'.", exit_status);
                    producer.send(Response::new(request.identifier, Ok(()))).unwrap();
                },
                Err(error) => {
                    debug!("Shell runner failed to execute (error: '{:?}').", error);
                    producer.send(Response::new(request.identifier, Err(()))).unwrap();
                },
            };
        });

        Ok(())
    }
}
