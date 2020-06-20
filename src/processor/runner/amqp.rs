use amiquip::{Connection, Publish};
use crate::execution::job::Job;
use crate::execution::Response;
use log::debug;
use std::sync::mpsc::Sender;
use std::thread;
use uuid::Uuid;

/// An execution request about a job paired with an AMQP runner.
#[derive(Debug)]
pub struct Request {
    identifier: Uuid,
    job: Job,
    dsn: String,
    exchange: String,
    routing_key: String,
}

impl Request {
    /// Create a new shell request.
    pub fn new(identifier: Uuid, job: Job, dsn: String, exchange: String, routing_key: String) -> Request {
        Request {
            identifier: identifier,
            job: job,
            dsn: dsn,
            exchange: exchange,
            routing_key: routing_key,
        }
    }
}

/// A runner publishing an AMQP message.
pub struct Amqp {}

impl Amqp {
    /// Execute the given job if the runner configuration is of type shell.
    pub fn execute(request: Request, producer: &Sender<Response>) -> Result<(), ()> {
        debug!("Executing {:?}.", request);
        let producer = producer.clone();

        thread::spawn(move || {
            let result = || -> Result<(), Error> {
                let mut connection = match Connection::insecure_open(&request.dsn) {
                    Ok(connection) => connection,
                    Err(_) => return Err(Error::ConnectionFailed),
                };
                let channel = match connection.open_channel(None) {
                    Ok(channel) => channel,
                    Err(_) => return Err(Error::ConnectionFailed),
                };
                let exchange = match channel.exchange_declare_passive(&request.exchange) {
                    Ok(exchange) => exchange,
                    Err(_) => return Err(Error::InvalidExchange),
                };
                if let Err(_) = exchange.publish(Publish::new(&request.job.get_identifier().as_bytes(), &request.routing_key)) {
                    return Err(Error::PublishingFailed);
                };
                debug!("AMQP runner successfully published {:?}.", &request);
                producer.send(Response::new(request.identifier, Ok(()))).unwrap();
                let _ = connection.close();

                Ok(())
            }();

            match result {
                Ok(_) => {},
                Err(error) => {
                    debug!("AMQP runner failed to publish {:?} ({:?}).", &request, &error);
                    producer.send(Response::new(request.identifier, Err(()))).unwrap();
                },
            };
        });

        Ok(())
    }
}

#[derive(Debug)]
enum Error {
    ConnectionFailed,
    InvalidExchange,
    PublishingFailed,
}
