use log::debug;
use std::process::Command;
use std::process::Stdio;
use crossbeam_channel::Receiver as CrossbeamReceiver;
use crossbeam_channel::Sender as CrossbeamSender;
use std::thread;
use uuid::Uuid;

pub type Sender = CrossbeamSender<Response>;
pub type Receiver = CrossbeamReceiver<Response>;

pub struct Response {
    pub identifier: Uuid,
    pub result: Result<(), ()>,
}

/// An execution request about a job paired with a shell runner.
#[derive(Debug)]
pub struct Request {
    identifier: Uuid,
    job_identifier: String,
    command: String,
}

impl Request {
    /// Create a new shell request.
    pub fn new(identifier: Uuid, job_identifier: String, command: String) -> Request {
        Request {
            identifier: identifier,
            job_identifier: job_identifier,
            command: command,
        }
    }
}

/// A runner executing a shell script with the job identifier as parameter.
pub struct Shell {}

impl Shell {
    /// Execute the given job if the runner configuration is of type shell.
    pub fn execute(request: Request, producer: &Sender) -> Result<(), ()> {
        debug!("Executing {:?}.", request);
        let producer = producer.clone();

        thread::spawn(move || {
            let mut command = Command::new("sh");

            command
                .arg(request.command)
                .arg(request.job_identifier)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
            ;
            match command.status() {
                Ok(exit_status) => {
                    debug!("Shell runner exiting with status '{:?}'.", exit_status);
                    producer.send(Response {
                        identifier: request.identifier,
                        result: Ok(()),
                    }).unwrap();
                },
                Err(error) => {
                    debug!("Shell runner failed to execute (error: '{:?}').", error);
                    producer.send(Response {
                        identifier: request.identifier,
                        result: Err(()),
                    }).unwrap();
                },
            };
        });

        Ok(())
    }
}
