mod job;
mod rule;

use crate::database::storage::Storage;
use crate::query::instruction::Instruction;
use job::Set as JobSet;
use rule::Set as RuleSet;
use chrono::DateTime;
use chrono::offset::Utc;

/// Match instructions with statically associated handlers, execute them and return operation
/// results.
pub struct Handler {}

impl Handler {
    /// Handle the given instruction and return the operation result.
    pub fn handle(instruction: &Instruction, current_datetime: &DateTime<Utc>, storage: &mut Storage) -> Result<(), ()> {
        match instruction {
            Instruction::Set { identifier, execution } => JobSet::handle(identifier, execution, current_datetime, storage),
            Instruction::RuleSet { identifier, pattern, runner } => RuleSet::handle(identifier, pattern, runner, current_datetime, storage),
        }
    }
}
