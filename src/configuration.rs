//! Kairoi's main configuration, loading the user's runtime configuration.
//!
//! The user's runtime configuration is loaded from the file `configuration.toml`, relative to the
//! executable directory. Every configuration option has a default value, allowing to start the
//! server without a configuration file.

use config::Config;
use config::ConfigError;
use config::File;
use config::FileFormat;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Log {
    #[serde(default)]
    pub level: LogLevel,
}

#[derive(Debug, Deserialize)]
pub struct ControllerListen (String);
impl ToString for ControllerListen {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
impl Default for ControllerListen {
    fn default() -> Self {
        Self ("127.0.0.1:5678".to_string())
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Controller {
    #[serde(default)]
    pub listen: ControllerListen,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Database {
    pub fsync_on_persist: bool,
}
impl Default for Database {
    fn default() -> Self {
        Self {
            fsync_on_persist: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(default)]
    pub log: Log,
    #[serde(default)]
    pub controller: Controller,
    #[serde(default)]
    pub database: Database,
}

impl Configuration {
    /// Load the configuration from the `configuration.toml` file. It returns a properly
    /// instantiated configuration tree in case of success, or a message describing the error in
    /// case of error.
    pub fn new() -> Result<Self, String> {
        match Self::load() {
            Ok(configuration) => Ok(configuration),
            Err(error) => Err(error.to_string()),
        }
    }

    fn load() -> Result<Self, ConfigError> {
        let mut configuration = Config::default();

        let file =
            File::with_name("configuration.toml")
            .format(FileFormat::Toml)
            .required(false)
        ;
        configuration.merge(file)?;

        configuration.try_into()
    }
}
