use chrono::DateTime;
use chrono::offset::Utc;
use crate::database::storage::{Job, JobStatus, Storage};
use log::debug;

/// Handle Job Set instructions.
pub struct Set {}

impl Set {
    /// Register a Job with the given identifier and execution time to the given context.
    pub fn handle(identifier: &String, execution: &DateTime<Utc>, current_datetime: &DateTime<Utc>, storage: &mut Storage) -> Result<(), ()> {
        let job = Job::new(
            identifier.clone(),
            *execution,
            JobStatus::Planned,
        );
        // Check if the entry exists.
        match storage.get_job(identifier) {
            Some(current) => {
                // If the status is Planned, Executed or Failed, we can modify the job.
                match current.get_status() {
                    JobStatus::Triggered => {
                        debug!("Unable to SET {:?} at {} (in status Triggered).", &job, current_datetime);

                        Err(())
                    },
                    _ => {
                        debug!("SET {:?} at {}.", &job, current_datetime);

                        match storage.set_job(job) {
                            Ok(_) => Ok(()),
                            Err(_) => Err(()),
                        }
                    },
                }
            },
            None => {
                // We insert this new job.
                debug!("SET {:?} at {}.", &job, current_datetime);

                match storage.set_job(job) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(()),
                }
            },
        }
    }
}
