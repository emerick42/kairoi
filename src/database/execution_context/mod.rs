mod job;

use chrono::DateTime;
use chrono::offset::Utc;
use job::{Job as UnderlyingJob, Status as UnderlyingJobStatus, Storage as JobStorage};
use std::collections::HashMap;
use super::rule::Rule;

pub type JobStatus = UnderlyingJobStatus;
pub type Job = UnderlyingJob;

/// A database execution context, memorizing all existing jobs and rules.
pub struct ExecutionContext {
    job_storage: JobStorage,
    rules: HashMap<String, Rule>,
    current_datetime: DateTime<Utc>,
}

impl ExecutionContext {
    /// Create a new execution context.
    pub fn new() -> ExecutionContext {
        ExecutionContext {
            job_storage: JobStorage::new(),
            rules: HashMap::new(),
            current_datetime: Utc::now(),
        }
    }

    /// Update the current datetime of this execution context, to be the more accurate possible.
    pub fn update_clock(&mut self) {
        self.current_datetime = Utc::now();
    }

    /// Get the current datetime of this execution context.
    pub fn get_current_datetime(&self) -> DateTime<Utc> {
        self.current_datetime
    }

    /// Get all jobs that need to be executed.
    pub fn get_jobs_to_execute(&self) -> Vec<&Job> {
        self.job_storage.get_to_execute(&self.current_datetime)
    }

    /// Get the job with the given identifier, if there is one.
    pub fn get_job(&self, identifier: &str) -> Option<&Job> {
        self.job_storage.get(identifier)
    }

    /// Set a job in this execution context. If a job with the same identifier already exists,
    /// update its properties.
    pub fn set_job(&mut self, job: Job) {
        self.job_storage.set(job);
    }

    /// Set a rule in this execution context. If a rule with the same identifier already exists,
    /// update its properties.
    pub fn set_rule(&mut self, rule: Rule) {
        self.rules.insert(rule.get_identifier().clone(), rule);
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
