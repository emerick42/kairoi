//! Kairoi's global logging system.
//!
//! Kairoi uses the standard [log](https://docs.rs/log/0.4.14/log/) crate, with the
//! [simple_logger](https://docs.rs/simple_logger/1.13.0/simple_logger/) implementation, to enable
//! logging in the entire application.
//!
//! This module only provides an encapsulation of their initialization.

use log::Level as LogLevel;

pub enum Level {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

pub struct Logger {}

impl Logger {
    /// Initialize the global logger with the given level. In case of failure, this method panicks.
    /// It must only be used once, since all subsequent calls will result in a failure.
    pub fn initialize(level: Level) {
        match level {
            Level::Off => {},
            Level::Error => simple_logger::init_with_level(LogLevel::Error).unwrap(),
            Level::Warn => simple_logger::init_with_level(LogLevel::Warn).unwrap(),
            Level::Info => simple_logger::init_with_level(LogLevel::Info).unwrap(),
            Level::Debug => simple_logger::init_with_level(LogLevel::Debug).unwrap(),
            Level::Trace => simple_logger::init_with_level(LogLevel::Trace).unwrap(),
        };
    }
}
