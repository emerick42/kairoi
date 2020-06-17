use std::sync::mpsc::{channel, Receiver, Sender};

/// Create a new bidirectionnal channel, with the owning side being able to produce messages, and
/// the reverse side being able to asynchronously confirm these messages.
pub fn link<T, U>() -> ((Sender<T>, Receiver<U>), (Sender<U>, Receiver<T>)) {
    let (producer1, consumer1) = channel();
    let (producer2, consumer2) = channel();

    ((producer1, consumer2), (producer2, consumer1))
}
