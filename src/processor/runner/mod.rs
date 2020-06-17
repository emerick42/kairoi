pub mod shell;

use crate::execution::{Request, Response};
use std::sync::mpsc::Sender;

/// A runner, asynchronously executing jobs according to runner configurations.
pub trait Runner {
    /// Execute the given request, using the given producer to send the execution response
    /// asynchronously to the calling processor.
    fn execute(&self, request: &Request, producer: &Sender<Response>) -> Result<(), ()>;
}
