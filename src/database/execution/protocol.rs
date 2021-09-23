//! Protocol implementation for the communication between the execution client and the processor.
//!
//! It defines the [`Request`] and the [`Response`], respectively used to request an execution to
//! the processor, and to retrieve the execution result once the processor has handled it.

use uuid::Uuid;

#[derive(Clone)]
pub struct Request {
    pub identifier: Uuid,
    pub job_identifier: String,
    pub runner: Runner,
}

pub struct Response {
    pub identifier: Uuid,
    pub result: Result<(), ()>
}

#[derive(Clone)]
pub enum Runner {
    Shell {
        command: String,
    },
    Amqp {
        dsn: String,
        exchange: String,
        routing_key: String,
    },
}
