use chrono::DateTime;
use chrono::offset::Utc;
use crate::database::storage::{Rule, Runner, Storage};
use crate::execution::runner::Runner as ExecutionRunner;
use log::debug;

/// Handle Rule Set instructions.
pub struct Set {}

impl Set {
    /// Register a Rule with the given identifier, pattern and runner configuration to the given
    /// execution context.
    pub fn handle(identifier: &str, pattern: &str, runner: &ExecutionRunner, current_datetime: &DateTime<Utc>, storage: &mut Storage) -> Result<(), ()> {
        let rule = Rule::new(identifier.to_string(), pattern.to_string(), Runner::from(runner.clone()));
        debug!("RULE SET {:?} at {}.", &rule, current_datetime);

        match storage.set_rule(rule) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}

/// Convert ExecutionRunner into Runner.
impl From<ExecutionRunner> for Runner {
    fn from(runner: ExecutionRunner) -> Self {
        match runner {
            ExecutionRunner::Amqp { dsn, exchange, routing_key } => Self::Amqp { dsn, exchange, routing_key },
            ExecutionRunner::Shell { command } => Self::Shell { command },
        }
    }
}
