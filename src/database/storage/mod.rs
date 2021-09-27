mod job;
mod rule;
mod persistence;

use chrono::{DateTime, offset::Utc};
use log::{debug, error};
use self::job::{Storage as JobStorage};
use self::persistence::{Entry, Job as PersistentJob, JobStatus as PersistentJobStatus, Rule as PersistentRule, Runner as PersistentRunner, Storage as PersistentStorage};
use std::collections::HashMap;

pub type JobStatus = job::Status;
pub type Job = job::Job;
pub type Rule = rule::Rule;
pub type Runner = rule::Runner;
pub enum InitializeError {
    UninitializablePersistentStorage,
}
pub type InitializeResult = Result<Vec<Job>, InitializeError>;
pub enum WriteError {
    PersistenceFailure,
}
pub type WriteResult = Result<(), WriteError>;

/// A database Storage, memorizing all existing jobs and rules.
///
/// While the storage itself is in-memory, it encapsulates a persistent storage, making sure data
/// are synchronously written to the file system, so there is no data lost on system failure.
pub struct Storage {
    job_storage: JobStorage,
    rules: HashMap<String, Rule>,
    persistent_storage: PersistentStorage,
}

impl Storage {
    /// Create a new Storage.
    pub fn new() -> Storage {
        Storage {
            job_storage: JobStorage::new(),
            rules: HashMap::new(),
            persistent_storage: PersistentStorage::new(),
        }
    }

    /// Initialize this storage with data from the persistent storage. It returns all jobs that are
    /// in the `triggered` state (which is an temporary state, and thus should be resumed).
    pub fn initialize(&mut self) -> InitializeResult {
        debug!("Initialization started with persisted data.");

        let entries = match self.persistent_storage.initialize() {
            Ok(entries) => entries,
            Err(_) => return Err(InitializeError::UninitializablePersistentStorage),
        };

        debug!("Reconstructing the in-memory storage from all {:?} persisted entries.", entries.len());
        let mut triggered = Vec::new();
        for entry in entries {
            match entry {
                Entry::Job(job) => {
                    let job = Job::from(job);
                    self.job_storage.set(job.clone());

                    if *job.get_status() == JobStatus::Triggered {
                        triggered.push(job);
                    }
                },
                Entry::Rule(rule) => {
                    self.rules.insert(rule.identifier.clone(), Rule::from(rule));
                },
            };
        };

        debug!("Initialization properly done with persisted data.");

        Ok(triggered)
    }

    /// Get all jobs that need to be executed at the given datetime.
    pub fn get_jobs_to_execute(&self, datetime: &DateTime<Utc>) -> Vec<Job> {
        self.job_storage.get_to_execute(datetime)
    }

    /// Get the job with the given identifier, if there is one.
    pub fn get_job(&self, identifier: &str) -> Option<&Job> {
        self.job_storage.get(identifier)
    }

    /// Set a job in this execution context. If a job with the same identifier already exists,
    /// update its properties.
    pub fn set_job(&mut self, job: Job) -> WriteResult {
        match self.persistent_storage.persist(Entry::Job(PersistentJob::from(job.clone()))) {
            Ok(_) => {
                self.job_storage.set(job);

                Ok(())
            },
            Err(_) => {
                error!("Unable to persist the job {:?} to the storage.", &job);

                Err(WriteError::PersistenceFailure)
            },
        }
    }

    /// Set a rule in this execution context. If a rule with the same identifier already exists,
    /// update its properties.
    pub fn set_rule(&mut self, rule: Rule) -> WriteResult {
        match self.persistent_storage.persist(Entry::Rule(PersistentRule::from(rule.clone()))) {
            Ok(_) => {
                self.rules.insert(rule.get_identifier().clone(), rule);

                Ok(())
            },
            Err(_) => {
                error!("Unable to persist the rule {:?} to the storage.", &rule);

                Err(WriteError::PersistenceFailure)
            }
        }
    }

    /// Pair the job with the given identifier to a matching rule.
    pub fn pair(&self, job: &String) -> Option<Rule> {
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
            Some((_, rule)) => Some(rule.clone()),
            None => None,
        }
    }
}

/// Convert PersistentJob into Job.
impl From<PersistentJob> for Job {
    fn from(job: PersistentJob) -> Self {
        Self::new(
            job.identifier,
            job.execution,
            match job.status {
                PersistentJobStatus::Planned => JobStatus::Planned,
                PersistentJobStatus::Triggered => JobStatus::Triggered,
                PersistentJobStatus::Executed => JobStatus::Executed,
                PersistentJobStatus::Failed => JobStatus::Failed,
            },
        )
    }
}

/// Convert Job into PersistentJob.
impl From<Job> for PersistentJob {
    fn from(job: Job) -> Self {
        Self {
            identifier: job.get_identifier().clone(),
            execution: job.get_execution().clone(),
            status: match job.get_status() {
                JobStatus::Planned => PersistentJobStatus::Planned,
                JobStatus::Triggered => PersistentJobStatus::Triggered,
                JobStatus::Executed => PersistentJobStatus::Executed,
                JobStatus::Failed => PersistentJobStatus::Failed,
            }
        }
    }
}

/// Convert PersistentRule into Rule.
impl From<PersistentRule> for Rule {
    fn from(rule: PersistentRule) -> Self {
        Self::new(
            rule.identifier,
            rule.pattern,
            match rule.runner {
                PersistentRunner::Amqp { dsn, exchange, routing_key } => Runner::Amqp { dsn, exchange, routing_key },
                PersistentRunner::Shell { command } => Runner::Shell { command },
            },
        )
    }
}

/// Convert Rule into PersistentRule.
impl From<Rule> for PersistentRule {
    fn from(rule: Rule) -> Self {
        Self {
            identifier: rule.get_identifier().clone(),
            pattern: rule.get_pattern().clone(),
            runner: match rule.get_runner().clone() {
                Runner::Amqp { dsn, exchange, routing_key } => PersistentRunner::Amqp { dsn, exchange, routing_key },
                Runner::Shell { command } => PersistentRunner::Shell { command },
            },
        }
    }
}
