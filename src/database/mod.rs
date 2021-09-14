mod execution;
mod framerate;
mod query;
mod storage;

use framerate::Clock;
use chrono::offset::Utc;
use crate::execution::{Request as ExecutionRequest, Response as ExecutionResponse};
use crate::query::{Request as QueryRequest, Response as QueryResponse};
use self::execution::Handler as ExecutionHandler;
use self::query::Handler as QueryHandler;
use self::storage::Storage;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

/// Start the Database, spawning a thread and returning the join handle.
///
/// The Database is one of the 3 mains components of the Kairoi architecture. It uses channels to
/// receive queries from the Controller, to send query responses to the Controller, to send
/// execution requests to the Processor, and to receive execution responses from the Processor. The
/// Database runs in its own process, at its own framerate. The job and rule storage is delegated
/// to a [`Storage`]. The [`Storage`] is initialized at the start of the Database process.
pub fn start(
    query_link: (Sender<QueryResponse>, Receiver<QueryRequest>),
    execution_link: (Sender<ExecutionRequest>, Receiver<ExecutionResponse>),
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut storage = Storage::new();
        let mut execution_handler = ExecutionHandler::new(execution_link);
        let mut query_handler = QueryHandler::new(query_link);

        if let Err(_) = storage.initialize() {
            panic!("Unable to initialize the storage from data persisted to the file system.");
        };

        let clock = Clock::with_framerate(128);
        clock.start(|| {
            let current_datetime = Utc::now();
            query_handler.handle(&current_datetime, &mut storage);
            execution_handler.handle(&current_datetime, &mut storage);
        });
    })
}
