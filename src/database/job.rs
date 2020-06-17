use chrono::DateTime;
use chrono::offset::Utc;

/// The status of a job, either Planned, Triggered, Executed or Failed.
///
/// A job can have the following statuses:
/// - Planned: the job is planned for execution.
/// - Triggered: the job execution is triggered and an execution request is being sent to the processor.
/// - Executed: the processor has confirmed that the job has been properly executed.
/// - Failed: the processor has confirmed that the job has failed to be executed.
/// A job status can only change from Planned to Triggered or Failed, from Triggered to Executed or
/// Failed, and from Executed or Failed to Planned.
#[derive(Debug, Copy, Clone)]
pub enum Status {
    Planned,
    Triggered,
    Executed,
    Failed,
}

/// A job, executed at some point in the time.
#[derive(Clone, Debug)]
pub struct Job {
    identifier: String,
    execution: DateTime<Utc>,
    status: Status,
}

impl Job {
    /// Create a new job.
    pub fn new(identifier: String, execution: DateTime<Utc>) -> Job {
        Job {
            identifier: identifier,
            execution: execution,
            status: Status::Planned,
        }
    }

    /// Get the identifier of this job.
    pub fn get_identifier(&self) -> &String {
        &self.identifier
    }

    /// Get the execution datetime of this job.
    pub fn get_execution(&self) -> &DateTime<Utc> {
        &self.execution
    }

    /// Set the execution datetime of this job.
    pub fn set_execution(&mut self, execution: DateTime<Utc>) {
        self.execution = execution;
    }

    /// Get the current status of this job.
    pub fn get_status(&self) -> &Status {
        &self.status
    }

    /// Set the status of this job to Triggered. Return Ok if the job was in status Planned, Err
    /// otherwise.
    pub fn set_triggered(&mut self) -> Result<(), ()> {
        match self.status {
            Status::Planned => {
                self.status = Status::Triggered;
                Ok(())
            },
            _ => Err(()),
        }
    }

    /// Set the status of this job to Executed. Return Ok if the job was in status Triggered, and
    /// Err otherwise.
    pub fn set_executed(&mut self) -> Result<(), ()> {
        match self.status {
            Status::Triggered => {
                self.status = Status::Executed;
                Ok(())
            },
            _ => Err(()),
        }
    }

    /// Set the status of this job to Failed. Return Ok if the job was in status Planned or
    /// Triggered, and Err otherwise.
    pub fn set_failed(&mut self) -> Result<(), ()> {
        match self.status {
            Status::Planned | Status::Triggered => {
                self.status = Status::Failed;
                Ok(())
            },
            _ => Err(()),
        }
    }

    /// Set the status of this job to Planned. Return Ok if the job was in status Executed or
    /// Failed, and Err otherwise.
    pub fn set_planned(&mut self) -> Result<(), ()> {
        match self.status {
            Status::Executed | Status::Failed => {
                self.status = Status::Planned;
                Ok(())
            },
            _ => Err(()),
        }
    }
}
