use chrono::DateTime;
use chrono::offset::Utc;
use crate::database::job::Job;
use std::collections::HashMap;
use super::rule::Rule;

/// A database execution context, memorizing all existing jobs and rules.
pub struct ExecutionContext {
    jobs: HashMap<String, Job>,
    rules: HashMap<String, Rule>,
    current_datetime: DateTime<Utc>,
}

impl ExecutionContext {
    /// Create a new execution context.
    pub fn new() -> ExecutionContext {
        ExecutionContext {
            jobs: HashMap::new(),
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

    /// Get the mutable job with the given identifier.
    pub fn get_job_mut(&mut self, identifier: &str) -> Option<&mut Job> {
        self.jobs.get_mut(identifier)
    }

    /// Get all known jobs.
    pub fn get_jobs(&self) -> Vec<&Job> {
        self.jobs.values().collect()
    }

    /// Set a job in this execution context. If a job with the same identifier already exists,
    /// update its properties.
    pub fn set_job(&mut self, job: Job) {
        self.jobs.insert(job.get_identifier().clone(), job);
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
