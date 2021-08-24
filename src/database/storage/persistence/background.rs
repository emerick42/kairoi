use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread::spawn;

pub enum Status {
    Success,
    Failure(TaskError),
    Running,
    Lost,
}
pub enum TaskError {
    Failure,
}
pub type TaskResult = Result<(), TaskError>;

/// A process execute a custom task in background.
pub struct Process {
    receiver: Receiver<TaskResult>,
}

impl Process {
    /// Execute the given task in background. The task must return a [`TaskResult`], that will be
    /// asynchronously retrievable through the [`status`] method. This method returns a [`Process`]
    /// that should be used to regularly check the status of the task.
    pub fn execute<T>(task: T) -> Self
    where
        T: FnOnce() -> TaskResult,
        T: Send + 'static,
    {
        let (sender, receiver) = channel();

        spawn(move || {
            let result = task();

            // If the thread fails to send a notification, we should just terminate it.
            sender.send(result).unwrap();
        });

        Self {
            receiver: receiver,
        }
    }

    /// Check the status of the executed task.
    pub fn status(&self) -> Status {
        match self.receiver.try_recv() {
            Ok(result) => match result {
                Ok(_) => Status::Success,
                Err(error) => Status::Failure(error),
            },
            Err(error) => match error {
                TryRecvError::Empty => Status::Running,
                TryRecvError::Disconnected => Status::Lost,
            },
        }
    }
}
