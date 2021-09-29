mod client;

use client::Client;
use crate::query::{Request, Response};
use std::collections::HashMap;
use std::io;
use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

pub struct Controller {}

impl Controller {
    /// Start the controller, spawning a thread and returning the join handle. The given listen
    /// parameter should be a listenable address, including the port (for example
    /// `127.0.0.1:5678`), otherwise the thread will panic.
    pub fn start(listen: String, query_link: (Sender<Request>, Receiver<Response>)) -> thread::JoinHandle<()> {
        thread::Builder::new().name("kairoi/ctrl".to_string()).spawn(move || {
            let mut clients = HashMap::new();
            let mut identifier: u128 = 0;

            let server = TcpListener::bind(&listen).unwrap();
            server.set_nonblocking(true).unwrap();

            log::info!("Waiting for connections on {}.", &server.local_addr().unwrap());

            loop {
                let previous_time = Instant::now();

                // Accept all incoming connections.
                loop {
                    match server.accept() {
                        Ok(stream) => {
                            // @TODO: Handle the connection.
                            let (producer, consumer) = mpsc::channel();
                            clients.insert(identifier, producer);
                            Client::spawn(identifier, stream.0, query_link.0.clone(), consumer);
                            identifier += 1;
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        },
                        Err(error) => panic!("Encountered IO error: {}", error),
                    };
                }

                // Pull all received confirmation messages.
                loop {
                    match query_link.1.try_recv() {
                        Ok(response) => {
                            // Dispatch the result to the corresponding client.
                            let client = response.get_request().get_client();
                            let result = match clients.get_mut(&client) {
                                Some(producer) => {
                                    match producer.send(response) {
                                        Ok(_) => Ok(()),
                                        Err(_) => {
                                            log::debug!("Removing client {} (thread ended).", client);
                                            clients.remove(&client);

                                            Err(())
                                        },
                                    }
                                },
                                None => Err(()),
                            };
                            if let Err(_) = result {
                                log::debug!("[controller] Unable to notify the client {} for a response (client disconnected).", client);
                            };
                        },
                        Err(error) => match error {
                            TryRecvError::Empty => break,
                            TryRecvError::Disconnected => panic!("Query channel disconnected."),
                        },
                    }
                }

                // Put the thread asleep to run at a maximum of 128 time per second.
                let now = Instant::now();
                let elapsed_time = now.duration_since(previous_time);
                match Duration::new(0, 1_000_000_000u32 / 128).checked_sub(elapsed_time) {
                    Some(sleep_time) => {
                        thread::sleep(sleep_time);
                    },
                    None => {},
                };
            }
        }).unwrap()
    }
}
