extern crate chrono;
extern crate config;
extern crate crossbeam_channel;
extern crate log;
extern crate nom;
extern crate serde_derive;
extern crate serde;
extern crate simple_logger;

mod configuration;
mod controller;
mod database;
mod execution;
mod logger;
mod processor;
mod query;
mod sync;

use crossbeam_channel::select;
use crossbeam_channel::unbounded;
use self::configuration::Configuration;
use self::configuration::LogLevel as ConfigurationLogLevel;
use self::controller::Controller;
use self::database::Configuration as DatabaseConfiguration;
use self::database::Database;
use self::database::execution::protocol::Request as DatabaseExecutionRequest;
use self::database::execution::protocol::Response as DatabaseExecutionResponse;
use self::database::execution::protocol::Runner as DatabaseExecutionRunner;
use self::logger::Level as LoggerLevel;
use self::logger::Logger;
use self::processor::Processor;
use self::processor::protocol::Request as ProcessorExecutionRequest;
use self::processor::protocol::Response as ProcessorExecutionResponse;
use self::processor::protocol::Runner as ProcessorExecutionRunner;

fn main() {
    let configuration = match Configuration::new() {
        Ok(configuration) => configuration,
        Err(message) => {
            Logger::initialize(LoggerLevel::Error);
            log::error!("Unable to load the configuration file: {}.", message);

            return;
        },
    };

    Logger::initialize(LoggerLevel::from(configuration.log.level));
    log::debug!("Booting with the following configuration: {:?}.", &configuration);

    let (query_owning_side, query_reverse_side) = sync::link();
    let (database_execution_request_sender, execution_request_receiver) = unbounded();
    let (execution_request_sender, processor_execution_request_receiver) = unbounded();
    let (execution_response_sender, database_execution_response_receiver) = unbounded();
    let (processor_execution_response_sender, execution_response_receiver) = unbounded();

    // Spawn the controller, the database and the processor.
    Controller::start(configuration.controller.listen.to_string(), query_owning_side);
    Database::start(
        query_reverse_side,
        (database_execution_request_sender, database_execution_response_receiver),
        DatabaseConfiguration {
            storage_persistence_fsync_on_persist: configuration.database.fsync_on_persist,
        },
    );
    Processor::start((processor_execution_response_sender, processor_execution_request_receiver));

    loop {
        // There may be a more generic way to implement the message routing (maybe using traits?).
        // The current solution is working fine and is efficient in terms of runtime performances,
        // but it clearly is hard to maintain.
        select! {
            recv(execution_request_receiver) -> message => {
                match message {
                    Ok(message) => {
                        execution_request_sender.send(ProcessorExecutionRequest::from(message)).unwrap();
                    },
                    Err(_) => {
                        // @TODO: Handle the channel disconnection properly.
                        panic!("The Execution Request channel between the database and the processor has been disconnected.");
                    },
                };
            },
            recv(execution_response_receiver) -> message => {
                match message {
                    Ok(message) => {
                        execution_response_sender.send(DatabaseExecutionResponse::from(message)).unwrap();
                    },
                    Err(_) => {
                        // @TODO: Handle the channel disconnection properly.
                        panic!("The Execution Response channel between the processor and the database has been disconnected.");
                    },
                };
            },
        }
    }
}

impl From<DatabaseExecutionRequest> for ProcessorExecutionRequest {
    fn from(request: DatabaseExecutionRequest) -> Self {
        Self {
            identifier: request.identifier,
            job_identifier: request.job_identifier,
            runner: match request.runner {
                DatabaseExecutionRunner::Shell { command } => ProcessorExecutionRunner::Shell { command },
                DatabaseExecutionRunner::Amqp { dsn, exchange, routing_key } => ProcessorExecutionRunner::Amqp { dsn, exchange, routing_key },
            },
        }
    }
}

impl From<ProcessorExecutionResponse> for DatabaseExecutionResponse {
    fn from(response: ProcessorExecutionResponse) -> Self {
        Self {
            identifier: response.identifier,
            result: response.result,
        }
    }
}

impl From<ConfigurationLogLevel> for LoggerLevel {
    fn from(level: ConfigurationLogLevel) -> Self {
        match level {
            ConfigurationLogLevel::Off => LoggerLevel::Off,
            ConfigurationLogLevel::Error => LoggerLevel::Error,
            ConfigurationLogLevel::Warn => LoggerLevel::Warn,
            ConfigurationLogLevel::Info => LoggerLevel::Info,
            ConfigurationLogLevel::Debug => LoggerLevel::Debug,
            ConfigurationLogLevel::Trace => LoggerLevel::Trace,
        }
    }
}
