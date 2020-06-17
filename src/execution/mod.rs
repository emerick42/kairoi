pub mod job;
pub mod runner;

use job::Job;
use runner::Runner;
use uuid::Uuid;

/// An execution request for a job and an associated processor.
#[derive(Clone, Debug)]
pub struct Request {
    identifier: Uuid,
    job: Job,
    runner: Runner,
}

impl Request {
    /// Create a new trigger.
    pub fn new(identifier: Uuid, job: Job, runner: Runner) -> Request {
        Request {
            identifier: identifier,
            job: job,
            runner: runner,
        }
    }

    /// Get the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.identifier
    }

    /// Get the job.
    pub fn get_job(&self) -> &Job {
        &self.job
    }

    /// Get the processor.
    pub fn get_runner(&self) -> &Runner {
        &self.runner
    }
}

/// An execution response to a request, using the same shared identifier.
pub struct Response {
    identifier: Uuid,
    result: Result<(), ()>
}

impl Response {
    /// Create a new response to a request with the given identifier.
    pub fn new(identifier: Uuid, result: Result<(), ()>) -> Response {
        Response {
            identifier: identifier,
            result: result,
        }
    }

    /// Get the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.identifier
    }

    /// Get the result.
    pub fn get_result(&self) -> &Result<(), ()> {
        &self.result
    }
}
