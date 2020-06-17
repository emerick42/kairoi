#[derive(Clone, Debug)]
pub struct Job {
    identifier: String,
}

impl Job {
    /// Create a new job.
    pub fn new(identifier: String) -> Job {
        Job {
            identifier: identifier,
        }
    }

    /// Get the identifier.
    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }
}
