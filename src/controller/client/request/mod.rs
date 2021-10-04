pub mod job;
pub mod rule;

use crate::query::Client;
use crate::query::Request;
use crate::query::instruction::Instruction;

/// Build query requests from parsed arguments as an element of chain. Return a result when the
/// request is valid or when sub-arguments seem to contain errors. Otherwise, return nothing to let
/// another chainable builder handle these arguments.
pub trait Chainable {
    fn build(&self, arguments: &Vec<String>) -> Option<Result<Instruction, ()>>;
}

/// Build query requests from parsed arguments.
pub struct Builder {
    builders: Vec<Box<dyn Chainable>>,
}

impl Builder {
    /// Create a new builder.
    pub fn new(builders: Vec<Box<dyn Chainable>>) -> Builder {
        Builder {
            builders: builders,
        }
    }

    /// Build a query request from the given arguments.
    pub fn build(&self, client: &Client, identifier: &String, arguments: &Vec<String>) -> Result<Request, ()> {
        // Try all builders until a request is built.
        for builder in &self.builders {
            match builder.build(arguments) {
                Some(result) => match result {
                    Ok(instruction) => return Ok(Request::new(*client, identifier.clone(), instruction)),
                    Err(_) => return Err(()),
                },
                None => continue,
            };
        };

        Err(())
    }
}
