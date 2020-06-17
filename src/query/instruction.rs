use chrono::DateTime;
use chrono::offset::Utc;
use crate::execution::runner::Runner;

#[derive(Debug, Clone)]
pub enum Instruction {
    Set {
        identifier: String,
        execution: DateTime<Utc>,
    },
    RuleSet {
        identifier: String,
        pattern: String,
        runner: Runner,
    },
}
