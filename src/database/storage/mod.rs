mod encoder;
mod job;
mod write_ahead_logger;

use chrono::DateTime;
use chrono::offset::Utc;
use encoder::{Encoder, Encodable};
use job::{Job as UnderlyingJob, Status as UnderlyingJobStatus, Storage as JobStorage};
use log::error;
use std::collections::HashMap;
use super::rule::Rule as UnderlyingRule;
use write_ahead_logger::WriteAheadLogger;

pub type JobStatus = UnderlyingJobStatus;
pub type Job = UnderlyingJob;
pub type Rule = UnderlyingRule;

pub enum WriteError {
    EncodeFailure,
    WriteAheadLoggerFailure,
}
pub type WriteResult = Result<(), WriteError>;

/// A database Storage, memorizing all existing jobs and rules.
///
/// While the storage itself is in-memory, it encapsulates a write-ahead logger, making sure data
/// are synchronously written to the file system, so there is no data lost on system failure.
pub struct Storage {
    job_storage: JobStorage,
    rules: HashMap<String, Rule>,
    write_ahead_logger: WriteAheadLogger,
    encoder: Encoder,
}

impl Storage {
    /// Create a new Storage.
    pub fn new() -> Storage {
        Storage {
            job_storage: JobStorage::new(),
            rules: HashMap::new(),
            write_ahead_logger: WriteAheadLogger::new(),
            encoder: Encoder::new(),
        }
    }

    /// Get all jobs that need to be executed at the given datetime.
    pub fn get_jobs_to_execute(&self, datetime: &DateTime<Utc>) -> Vec<&Job> {
        self.job_storage.get_to_execute(datetime)
    }

    /// Get the job with the given identifier, if there is one.
    pub fn get_job(&self, identifier: &str) -> Option<&Job> {
        self.job_storage.get(identifier)
    }

    /// Set a job in this execution context. If a job with the same identifier already exists,
    /// update its properties.
    pub fn set_job(&mut self, job: Job) -> WriteResult {
        let entry = match self.encoder.encode(Encodable::Job(&job)) {
            Ok(entry) => entry,
            Err(_) => return Err(WriteError::EncodeFailure),
        };
        match self.write_ahead_logger.append(&entry) {
            Ok(_) => {
                self.job_storage.set(job);

                Ok(())
            },
            Err(_) => {
                error!("Unable to write the job {:?} to the storage.", &job);

                Err(WriteError::WriteAheadLoggerFailure)
            },
        }
    }

    /// Set a rule in this execution context. If a rule with the same identifier already exists,
    /// update its properties.
    pub fn set_rule(&mut self, rule: Rule) -> WriteResult {
        let entry = match self.encoder.encode(Encodable::Rule(&rule)) {
            Ok(entry) => entry,
            Err(_) => return Err(WriteError::EncodeFailure),
        };
        match self.write_ahead_logger.append(&entry) {
            Ok(_) => {
                self.rules.insert(rule.get_identifier().clone(), rule);

                Ok(())
            },
            Err(_) => {
                error!("Unable to write the rule {:?} to the storage.", &rule);

                Err(WriteError::WriteAheadLoggerFailure)
            },
        }
    }

    /// Pair the job with the given identifier to a matching rule.
    pub fn pair(&self, job: &String) -> Option<&Rule> {
        let mut prioritized_rule = None;

        for rule in self.rules.values() {
            match rule.supports(job) {
                Some(weight) => {
                    match prioritized_rule {
                        Some((prioritized_weight, _)) => if weight > prioritized_weight {
                            prioritized_rule = Some((weight, rule));
                        },
                        None => {
                            prioritized_rule = Some((weight, rule));
                        }
                    }
                }
                None => {},
            };
        };

        match prioritized_rule {
            Some((_, rule)) => Some(rule),
            None => None,
        }
    }
}
