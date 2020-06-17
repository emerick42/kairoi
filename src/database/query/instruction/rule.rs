use crate::database::execution_context::ExecutionContext;
use crate::database::rule::Rule;
use crate::execution::runner::Runner;
use log::debug;

/// Handle Rule Set instructions.
pub struct Set {}

impl Set {
    /// Register a Rule with the given identifier, pattern and runner configuration to the given
    /// execution context.
    pub fn handle(identifier: &str, pattern: &str, runner: &Runner, context: &mut ExecutionContext) -> Result<(), ()> {
        let rule = Rule::new(identifier.to_string(), pattern.to_string(), runner.clone());
        debug!("RULE SET {:?} at {}.", &rule, context.get_current_datetime());
        context.set_rule(rule);

        Ok(())
    }
}
