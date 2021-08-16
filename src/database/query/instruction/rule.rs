use chrono::DateTime;
use chrono::offset::Utc;
use crate::database::rule::Rule;
use crate::database::storage::Storage;
use crate::execution::runner::Runner;
use log::debug;

/// Handle Rule Set instructions.
pub struct Set {}

impl Set {
    /// Register a Rule with the given identifier, pattern and runner configuration to the given
    /// execution context.
    pub fn handle(identifier: &str, pattern: &str, runner: &Runner, current_datetime: &DateTime<Utc>, storage: &mut Storage) -> Result<(), ()> {
        let rule = Rule::new(identifier.to_string(), pattern.to_string(), runner.clone());
        debug!("RULE SET {:?} at {}.", &rule, current_datetime);

        match storage.set_rule(rule) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
