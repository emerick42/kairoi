use amiquip::Channel;
use amiquip::Connection;
use amiquip::Error as AmqpError;
use amiquip::Publish;
use crossbeam_channel::Receiver as CrossbeamReceiver;
use crossbeam_channel::Sender as CrossbeamSender;
use log::debug;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::VecDeque;
use uuid::Uuid;

pub type Sender = CrossbeamSender<Response>;
pub type Receiver = CrossbeamReceiver<Response>;

pub struct Response {
    pub identifier: Uuid,
    pub result: Result<(), ()>,
}

/// An execution request about a job paired with an AMQP runner.
#[derive(Debug)]
pub struct Request {
    identifier: Uuid,
    job_identifier: String,
    dsn: String,
    exchange: String,
    routing_key: String,
}

impl Request {
    /// Create a new shell request.
    pub fn new(identifier: Uuid, job_identifier: String, dsn: String, exchange: String, routing_key: String) -> Request {
        Request {
            identifier: identifier,
            job_identifier: job_identifier,
            dsn: dsn,
            exchange: exchange,
            routing_key: routing_key,
        }
    }
}

/// A runner publishing an AMQP message.
pub struct Amqp {
    client: Client,
}

impl Amqp {
    /// Create a new AMQP runner.
    pub fn new() -> Amqp {
        Amqp {
            client: Client::new(16),
        }
    }

    /// Execute the given job if the runner configuration is of type shell.
    pub fn execute(&mut self, request: Request, producer: &Sender) -> Result<(), ()> {
        debug!("Executing {:?}.", request);
        let producer = producer.clone();

        let result = || -> Result<(), Error> {
            let channel = match self.client.open(&request.dsn) {
                Ok((_, channel)) => channel,
                Err(error) => return Err(error),
            };
            let exchange = match channel.exchange_declare_passive(&request.exchange) {
                Ok(exchange) => exchange,
                Err(error) => {
                    return Err(Error::InvalidExchange(error));
                },
            };
            if let Err(error) = exchange.publish(Publish::new(&request.job_identifier.as_bytes(), &request.routing_key)) {
                return Err(Error::PublishingFailed(error));
            };
            debug!("AMQP runner successfully published {:?}.", &request);
            producer.send(Response {
                identifier: request.identifier,
                result: Ok(()),
            }).unwrap();

            Ok(())
        }();

        match result {
            Ok(_) => {},
            Err(error) => {
                debug!("AMQP runner failed to publish {:?} ({:?}).", &request, &error);
                self.client.drop(&request.dsn);
                producer.send(Response {
                    identifier: request.identifier,
                    result: Err(()),
                }).unwrap();
            },
        };

        Ok(())
    }
}

#[derive(Debug)]
enum Error {
    ConnectionFailed(AmqpError),
    InvalidExchange(AmqpError),
    PublishingFailed(AmqpError),
}

/// A client storing connections with AMPQ servers by data source name, droping older connections
/// each time a connection is initialized and the maximum memory capacity is reached.
pub struct Client {
    memory_capacity: usize,
    connections: HashMap<String, (Connection, Channel)>,
    dsns: VecDeque<String>,
}

impl<'a> Client {
    /// Create a new connection map with the given memory capacity.
    fn new(memory_capacity: usize) -> Client {
        Client {
            memory_capacity: memory_capacity,
            connections: HashMap::with_capacity(memory_capacity + 1),
            dsns: VecDeque::with_capacity(memory_capacity + 1),
        }
    }

    /// Open a connection with the given data source name, or retrieve an already open connection
    /// if there is one.
    fn open(&'a mut self, dsn: &String) -> Result<&'a mut (Connection, Channel), Error> {
        // Check if the maximum capacity is reached.
        if !self.connections.contains_key(dsn) && self.dsns.len() >= self.memory_capacity {
            // Remove the oldest entry.
            match self.dsns.pop_front() {
                Some(dsn) => {
                    self.connections.remove(&dsn);
                },
                None => {},
            }
        };

        let entry = self.connections.entry(dsn.clone());
        match entry {
            Entry::Occupied(entry) => {
                Ok(entry.into_mut())
            },
            Entry::Vacant(entry) => {
                let mut connection = match Connection::insecure_open(dsn) {
                    Ok(connection) => connection,
                    Err(error) => return Err(Error::ConnectionFailed(error)),
                };
                let channel = match connection.open_channel(None) {
                    Ok(channel) => channel,
                    Err(error) => return Err(Error::ConnectionFailed(error)),
                };

                self.dsns.push_back(dsn.clone());

                Ok(entry.insert((connection, channel)))
            },
        }
    }

    /// Drop the connection with the given data source name.
    fn drop(&mut self, dsn: &String) {
        self.connections.remove(dsn);

        self.dsns.retain(|element| *element == *dsn);
    }
}
