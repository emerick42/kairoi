mod execution_context;
mod execution;
mod query;
mod rule;

use crate::execution::{Request as ExecutionRequest, Response as ExecutionResponse};
use crate::query::{Request as QueryRequest, Response as QueryResponse};
use execution_context::ExecutionContext;
use execution::Handler as ExecutionHandler;
use query::Handler as QueryHandler;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

/// Start the database, spawning a thread and returning the join handle.
pub fn start(
    query_link: (Sender<QueryResponse>, Receiver<QueryRequest>),
    execution_link: (Sender<ExecutionRequest>, Receiver<ExecutionResponse>),
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut context = ExecutionContext::new();
        let mut execution_handler = ExecutionHandler::new(execution_link);
        let mut query_handler = QueryHandler::new(query_link);

        loop {
            let previous_time = Instant::now();

            context.update_clock();
            query_handler.handle(&mut context);
            execution_handler.handle(&mut context);

            // Put the thread asleep to run at a maximum of 128 time per second.
            let now = Instant::now();
            let elapsed_time = now.duration_since(previous_time);

            match Duration::new(0, 1_000_000_000u32 / 128).checked_sub(elapsed_time) {
                Some(sleep_time) => {
                    thread::sleep(sleep_time);
                },
                None => {},
            };
        };
    })
}
