use crate::query::instruction::Instruction;
use super::Chainable;
#[cfg(feature = "runner-shell")]
use crate::execution::runner::Runner;

/// Build Rule Set requests from parsed arguments.
pub struct Set {}

impl Set {
    /// Create a new Rule Set builder.
    pub fn new() -> Set {
        Set {}
    }
}

impl Chainable for Set {
    fn build(&self, arguments: &Vec<String>) -> Option<Result<Instruction, ()>> {
        // Handle all requests starting by "RULE SET".
        if arguments.len() < 2 || &arguments[0] != "RULE" || &arguments[1] != "SET" {
            return None
        };

        // If there is less than 5 arguments, it's an error.
        if arguments.len() < 5 {
            return Some(Err(()));
        };

        let identifier = &arguments[2];
        let pattern = &arguments[3];
        let runner = &arguments[4];

        match runner.as_str() {
            #[cfg(feature = "runner-shell")]
            "shell" => {
                if arguments.len() == 6 {
                    let command_line = &arguments[5];

                    Some(Ok(Instruction::RuleSet {
                        identifier: identifier.clone(),
                        pattern: pattern.clone(),
                        runner: Runner::Shell {
                            command: command_line.clone(),
                        },
                    }))
                } else {
                    Some(Err(()))
                }
            },
            _ => Some(Err(())),
        }
    }
}
