use chrono::DateTime;
use chrono::offset::Utc;
use crate::database::execution_context::ExecutionContext;
use log::debug;
use crate::database::job::{Job, Status};

/// Handle Job Set instructions.
pub struct Set {}

impl Set {
    /// Register a Job with the given identifier and execution time to the given context.
    pub fn handle(identifier: &String, execution: &DateTime<Utc>, context: &mut ExecutionContext) -> Result<(), ()> {
        let current_datetime = context.get_current_datetime();
        // Check if the entry exists.
        match context.get_job_mut(identifier) {
            Some(job) => {
                // If the status is Planned, Executed or Failed, we can modify the job.
                match job.get_status() {
                    Status::Triggered => {
                        debug!("Unable to SET {:?} at {} (in status Triggered).", Job::new(identifier.clone(), *execution), current_datetime);

                        Err(())
                    },
                    Status::Planned => {
                        debug!("SET {:?} at {}.", Job::new(identifier.clone(), *execution), current_datetime);
                        job.set_execution(*execution);

                        Ok(())
                    },
                    _ => {
                        debug!("SET {:?} at {}.", Job::new(identifier.clone(), *execution), current_datetime);
                        job.set_execution(*execution);
                        job.set_planned().unwrap();

                        Ok(())
                    },
                }
            },
            None => {
                // We insert this new job.
                debug!("SET {:?} at {}.", Job::new(identifier.clone(), *execution), current_datetime);
                context.set_job(Job::new(identifier.clone(), *execution));

                Ok(())
            },
        }
    }
}
