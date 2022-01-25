//! Kairoi's main configuration, loading the user's runtime configuration.
//!
//! The user's runtime configuration is loaded from either a given file, or from the default path
//! set at compilation time. The `CONFIGURATION_PATH` environment variable is read during
//! compilation. If it's empty, the default path is set to `configuration.toml`. Every
//! configuration option has a default value, allowing to start the server withtout a configuration
//! file.
//!
//! A configuration file's path can be either absolute or relative. In case of a relative path,
//! the path is computed starting from the launch directory.

use config::Config;
use config::ConfigError;
use config::File;
use config::FileFormat;
use serde::Deserialize;
use validator::Validate;

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

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Database {
    pub persistence: bool,
    pub fsync_on_persist: bool,
    #[validate(range(min = 1, max = 65535))]
    pub framerate: i64,
}
impl Default for Database {
    fn default() -> Self {
        Self {
            persistence: true,
            fsync_on_persist: true,
            framerate: 512,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(default)]
    pub log: Log,
    #[serde(default)]
    pub controller: Controller,
    #[serde(default)]
    #[validate]
    pub database: Database,
}

impl Configuration {
    /// Load the configuration from the default path, or the given path if one is given. It returns
    /// a properly  instantiated configuration tree in case of success, or a message describing the
    /// error in case of error.
    ///
    /// The default path is set at compilation time, by using the CONFIGURATION_PATH environment
    /// variable. When this variable is not set, it defaults to `configuration.toml`, loading the
    /// file relatively to the executable launch.
    pub fn new(configuration_path: Option<&str>) -> Result<Self, String> {
        match Self::load(configuration_path) {
            Ok(configuration) => {
                match configuration.validate() {
                    Ok(_) => {},
                    Err(error) => return Err(error.to_string()),
                };

                Ok(configuration)
            },
            Err(error) => Err(error.to_string()),
        }
    }

    fn load(configuration_path: Option<&str>) -> Result<Self, ConfigError> {
        let mut configuration = Config::default();

        let file =
            match configuration_path {
                Some(path) => File::with_name(path),
                None => match option_env!("CONFIGURATION_PATH") {
                    Some(path) => File::with_name(path),
                    None => File::with_name("configuration.toml"),
                },
            }
            .format(FileFormat::Toml)
            .required(false)
        ;
        configuration.merge(file)?;

        configuration.try_into()
    }
}
