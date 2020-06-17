use chrono::offset::{TimeZone, Utc};
use crate::query::instruction::Instruction;
use log::debug;
use super::Chainable;

/// Build Job Set requests from parsed arguments.
pub struct Set {}

impl Set {
    /// Create a new Job Set builder.
    pub fn new() -> Set {
        Set {}
    }
}

impl Chainable for Set {
    fn build(&self, arguments: &Vec<String>) -> Option<Result<Instruction, ()>> {
        let instruction = &arguments[0];

        if instruction == "SET" && arguments.len() == 3 {
            let identifier = &arguments[1];
            let execution = &arguments[2];
            let execution = match Utc.datetime_from_str(execution, "%F %T") {
                Ok(execution) => execution,
                Err(_) => {
                    debug!("Unable to build date from string {}.", execution);

                    return Some(Err(()))
                },
            };

            Some(Ok(Instruction::Set {
                identifier: identifier.clone(),
                execution: execution,
            }))
        } else {
            None
        }
    }
}
