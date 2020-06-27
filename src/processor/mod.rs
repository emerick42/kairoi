mod runner;

use crate::execution::{Request, Response};
use log::debug;
use runner::Runner;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

/// Start the processor, spawning a thread and returning the join handle.
pub fn start(execution_link: (Sender<Response>, Receiver<Request>)) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut runner = Runner::new();

        loop {
            let previous_time = Instant::now();

            // Pull all execution requests.
            let mut requests = Vec::with_capacity(100);
            loop {
                match execution_link.1.try_recv() {
                    Ok(request) => requests.push(request),
                    Err(error) => match error {
                        TryRecvError::Empty => break,
                        TryRecvError::Disconnected => panic!("Execution channel disconnected."),
                    },
                };
            };

            // Handle all received execution requests.
            for request in &requests {
                match runner.execute(request, &execution_link.0) {
                    Ok(_) => {},
                    Err(_) => {
                        debug!("Unable to execute {:?}.", request);
                        // Mark the job as failed.
                        let response = Response::new(*request.get_identifier(), Err(()));
                        if let Err(_) = execution_link.0.send(response) {
                            panic!("Execution channel disconnected.");
                        };
                    },
                };
            };

            // Put the thread asleep to run at a maximum of 128 time per second.
            let now = Instant::now();
            let elapsed_time = now.duration_since(previous_time);

            match Duration::new(0, 1_000_000_000u32 / 128).checked_sub(elapsed_time) {
                Some(sleep_time) => {
                    thread::sleep(sleep_time);
                },
                None => (),
            };
        };
    })
}
