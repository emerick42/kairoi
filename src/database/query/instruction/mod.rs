mod job;
mod rule;

use crate::database::execution_context::ExecutionContext;
use crate::query::instruction::Instruction;
use job::Set as JobSet;
use rule::Set as RuleSet;

/// Match instructions with statically associated handlers, execute them and return operation
/// results.
pub struct Handler {}

impl Handler {
    /// Handle the given instruction and return the operation result.
    pub fn handle(instruction: &Instruction, context: &mut ExecutionContext) -> Result<(), ()> {
        match instruction {
            Instruction::Set { identifier, execution } => JobSet::handle(identifier, execution, context),
            Instruction::RuleSet { identifier, pattern, runner } => RuleSet::handle(identifier, pattern, runner, context),
        }
    }
}
