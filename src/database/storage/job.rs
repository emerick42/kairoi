use chrono::DateTime;
use chrono::offset::Utc;
use std::collections::HashMap;

/// The status of a job, either Planned, Triggered, Executed or Failed.
///
/// A job can have the following statuses:
/// - Planned: the job is planned for execution.
/// - Triggered: the job execution is triggered and an execution request is being sent to the processor.
/// - Executed: the processor has confirmed that the job has been properly executed.
/// - Failed: the processor has confirmed that the job has failed to be executed.
/// A job status can only change from Planned to Triggered or Failed, from Triggered to Executed or
/// Failed, and from Executed or Failed to Planned.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Planned,
    Triggered,
    Executed,
    Failed,
}

/// A job, executed at some point in the time.
#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    identifier: String,
    execution: DateTime<Utc>,
    status: Status,
}

impl Job {
    /// Create a new job.
    pub fn new(identifier: String, execution: DateTime<Utc>, status: Status) -> Job {
        Job {
            identifier: identifier,
            execution: execution,
            status: status,
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

    /// Get the current status of this job.
    pub fn get_status(&self) -> &Status {
        &self.status
    }
}

/// An optimized storage implementation for jobs, aiming for fast reads on specific domain needs.
///
/// This storage provides access to all jobs "that must be executed" at a given date. Jobs can also
/// be retrieved directly using their identifiers. Finally, jobs can be set (creation or
/// modification) using their identifiers.
pub struct Storage {
    jobs: HashMap<String, Job>,
    to_execute: Vec<Job>,
}

impl Storage
{
    /// Create a new empty storage.
    pub fn new() -> Storage {
        Storage {
            jobs: HashMap::new(),
            to_execute: Vec::new(),
        }
    }

    /// Retrieve all jobs to be executed at the given current_datetime (included). Only retrieve
    /// jobs in the Planned status.
    pub fn get_to_execute(&self, current_datetime: &DateTime<Utc>) -> Vec<Job> {
        // Retrieve the position of the first element having an execution time beyond the current
        // time, then return all the previous elements of the to_execute vector.
        match self.to_execute.iter().position(|element| element.get_execution() > current_datetime) {
            Some(position) => {
                let mut result = Vec::with_capacity(position);

                for job in &self.to_execute[0..position] {
                    result.push(job.clone());
                };

                result
            },
            None => {
                let mut result = Vec::with_capacity(self.to_execute.len());

                for job in &self.to_execute[..] {
                    result.push(job.clone());
                };

                result
            },
        }
    }

    /// Retrieve the job with the given identifier, if there is one.
    pub fn get(&self, identifier: &str) -> Option<&Job> {
        self.jobs.get(identifier)
    }

    /// Set the given job, creating it if it doesn't exist, or modifying the entry with the same
    /// identifier to set the new properties.
    pub fn set(&mut self, job: Job) {
        match self.jobs.insert(job.get_identifier().clone(), job.clone()) {
            Some(old_value) => {
                // Remove the element in the ordered vector if it was planned.
                if *old_value.get_status() == Status::Planned {
                    self.to_execute.retain(|element| element.get_identifier() != old_value.get_identifier());
                };
            },
            None => {},
        };

        // Insert the new element in the ordered vector.
        if *job.get_status() == Status::Planned {
            match self.to_execute.iter().position(|element| element.get_execution() > job.get_execution()) {
                Some(position) => {
                    self.to_execute.insert(position, job);
                },
                None => {
                    self.to_execute.push(job);
                },
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::chrono::TimeZone;

    #[test]
    fn single_set_get() {
        let mut storage = Storage::new();
        let now = Utc::now();
        let job = Job::new(String::from("job"), now, Status::Planned);

        storage.set(job.clone());
        assert_eq!(
            storage.get(&"job"),
            Some(&job),
        );
    }

    #[test]
    fn get_to_execute() {
        let mut storage = Storage::new();
        let now = Utc.ymd(2020, 7, 24).and_hms(10, 32, 00);
        let datetime1 = Utc.ymd(2020, 7, 24).and_hms(10, 30, 00);
        let datetime2 = Utc.ymd(2020, 7, 24).and_hms(10, 31, 00);
        let datetime3 = Utc.ymd(2020, 7, 24).and_hms(10, 32, 00);
        let datetime4 = Utc.ymd(2020, 7, 24).and_hms(10, 33, 00);
        let job1 = Job::new(String::from("job.1"), datetime1, Status::Planned);
        let job2 = Job::new(String::from("job.2"), datetime2, Status::Executed);
        let job3 = Job::new(String::from("job.3"), datetime3, Status::Planned);
        let job4 = Job::new(String::from("job.4"), datetime4, Status::Planned);

        storage.set(job1.clone());
        storage.set(job2.clone());
        storage.set(job3.clone());
        storage.set(job4.clone());
        assert_eq!(
            storage.get_to_execute(&now),
            vec![&job1, &job3],
        );
    }
    #[test]
    fn get_to_execute_with_sequential_modifications() {
        let mut storage = Storage::new();
        let now = Utc.ymd(2020, 7, 24).and_hms(10, 32, 00);
        let datetime1 = Utc.ymd(2020, 7, 24).and_hms(10, 30, 00);
        let datetime2 = Utc.ymd(2020, 7, 24).and_hms(10, 31, 00);
        let datetime3 = Utc.ymd(2020, 7, 24).and_hms(10, 32, 00);
        let datetime4 = Utc.ymd(2020, 7, 24).and_hms(10, 33, 00);
        let job1 = Job::new(String::from("job.1"), datetime1, Status::Planned);
        let job2 = Job::new(String::from("job.2"), datetime2, Status::Executed);
        let job3 = Job::new(String::from("job.3"), datetime3, Status::Planned);
        let job4 = Job::new(String::from("job.4"), datetime4, Status::Planned);

        storage.set(job1.clone());
        storage.set(job2.clone());
        storage.set(job3.clone());
        storage.set(job4.clone());
        assert_eq!(
            storage.get_to_execute(&now),
            vec![&job1, &job3],
        );

        let job1 = Job::new(String::from("job.1"), datetime1, Status::Triggered);
        storage.set(job1.clone());
        assert_eq!(
            storage.get_to_execute(&now),
            vec![&job3],
        );
    }
}
