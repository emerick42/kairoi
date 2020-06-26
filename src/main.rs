extern crate chrono;
extern crate log;
extern crate nom;
extern crate simple_logger;

mod controller;
mod database;
mod execution;
mod processor;
mod query;
mod sync;

use log::Level;

fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let mut handles = vec![];

    let (query_owning_side, query_reverse_side) = sync::link();
    let (notification_owning_side, notification_reverse_side) = sync::link();

    // Spawn the controller, the database and the processor.
    handles.push(controller::start(query_owning_side));
    handles.push(database::start(query_reverse_side, notification_owning_side));
    handles.push(processor::start(notification_reverse_side));

    // Wait for all threads to finish.
    for handle in handles {
        let _ = handle.join();
    }
}
