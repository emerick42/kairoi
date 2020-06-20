mod parser;
mod request;

use crate::query::{Request, Response};
use crate::query::Client as ClientIdentifier;
use log::debug;
use parser::{Error, parse};
use request::Builder;
use request::Chainable;
use request::job::Set as JobSet;
use request::rule::Set as RuleSet;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub struct Client {}

impl Client {
    /// Spawn a new thread, creating a client with the given identifier to handle the given stream.
    /// Use the given producer to send request to the database, and receive confirmations on the
    /// given consumer.
    pub fn spawn(identifier: ClientIdentifier, mut stream: TcpStream, producer: Sender<Request>, consumer: Receiver<Response>) -> () {
        thread::spawn(move || {
            stream.set_nonblocking(false).unwrap();
            let builders_chain: Vec<Box<dyn Chainable>> = vec![Box::new(JobSet::new()), Box::new(RuleSet::new())];
            let builder = Builder::new(builders_chain);
            let mut input = String::new();
            let mut bytes_to_parse: Option<Vec<u8>> = None;

            loop {
                loop {
                    // Try to retrieve a request from the given input.
                    match parse(&input) {
                        Ok((input_left, arguments)) => {
                            input = input_left.to_string();
                            // Construct a request from the given arguments and send it to the database.
                            debug!("Reading input {:?} from client {}.", &arguments, identifier);
                            match builder.build(&identifier, &arguments) {
                                Ok(request) => {
                                    debug!("Sending {:?} to the database.", &request);
                                    if let Err(_) = producer.send(request) {
                                        panic!("Database channel disconnected.");
                                    };
                                    break;
                                },
                                Err(_) => {
                                    // Send an error response to the client.
                                    debug!("Invalid input {:?} from client {}.", &arguments, identifier);
                                    match stream.write_all("ERROR\n".as_bytes()) {
                                        Ok(_) => continue,
                                        Err(_) => panic!("An unexpected error occurred while writing a client response."),
                                    };
                                },
                            };
                        },
                        Err((_, error)) if error == Error::Incomplete => {},
                        Err(_) => panic!("An unexpected error occurred while handling a client request."),
                    };
                    // Read more data from the stream of the connected client.
                    let mut buffer = [0; 2048];
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            debug!("EOF reached for client {}.", identifier);
                            return;
                        },
                        Ok(length) => {
                            let buffer = match bytes_to_parse {
                                Some(bytes_to_parse) => {
                                    let mut copied_buffer = vec![0; bytes_to_parse.len() + length];
                                    &copied_buffer[0..bytes_to_parse.len()].copy_from_slice(&bytes_to_parse);
                                    &copied_buffer[bytes_to_parse.len()..bytes_to_parse.len() + length].copy_from_slice(&buffer[0..length]);

                                    copied_buffer
                                },
                                None => {
                                    let mut copied_buffer = vec![0; length];
                                    &copied_buffer[..].copy_from_slice(&buffer[..length]);

                                    copied_buffer
                                },
                            };
                            let (output, left_bytes) = Client::from_utf8_lossy(&buffer);
                            input.push_str(&output);
                            match left_bytes {
                                Some(left_bytes) => {
                                    bytes_to_parse = Some(left_bytes.to_vec());
                                },
                                None => {
                                    bytes_to_parse = None;
                                },
                            };
                        },
                        Err(ref error) if error.kind() == ErrorKind::Interrupted => continue,
                        Err(_) => panic!("An unexpected error occurred while reading a client request."),
                    };
                    // Continue reading until an instruction is produced.
                }

                // Pull the instruction confirmation and send a response.
                match consumer.recv() {
                    Ok(response) => {
                        debug!("Sending {:?} to client {}.", &response, identifier);
                        let output = match response.get_result() {
                            Ok(_) => String::from("OK\n"),
                            Err(_) => String::from("ERROR\n"),
                        };
                        match stream.write_all(output.as_bytes()) {
                            Ok(_) => {},
                            Err(_) => panic!("An unexpected error occurred while writing a client response."),
                        };
                    },
                    Err(_) => panic!("Database channel disconnected."),
                }
            }
        });
    }

    /// Parse the given input as utf8. Return the parsed utf8 String, and bytes left to parse if
    /// there are any.
    fn from_utf8_lossy(mut input: &[u8]) -> (String, Option<&[u8]>) {
        let mut output = String::new();

        loop {
            match std::str::from_utf8(input) {
                Ok(valid) => {
                    output.push_str(valid);

                    break (output, None)
                }
                Err(error) => {
                    let (valid, after_valid) = input.split_at(error.valid_up_to());
                    unsafe {
                        output.push_str(std::str::from_utf8_unchecked(valid))
                    }
                    output.push(std::char::REPLACEMENT_CHARACTER);

                    if let Some(invalid_sequence_length) = error.error_len() {
                        input = &after_valid[invalid_sequence_length..]
                    } else {
                        break (output, Some(after_valid))
                    }
                }
            }
        }
    }
}
