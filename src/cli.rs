//! Kairoi's CLI management, parsing CLI arguments given by the user.
//!
//! This module provides a complete handling on CLI arguments, including parsing arguments given by
//! the user, but also displaying the help and version commands. When arguments cannot get parsed,
//! it exits the program, displaying an appropriate message and returning the proper error code to
//! the parent shell.
//!
//! Currently, there is a single argument handles by this module:
//! * `-c`, `--config`: it takes a value as parameter, being the path to the configuration file
//! used for the current execution.

use clap::App;
use clap::crate_name;
use clap::crate_version;
use clap::Arg;

pub struct Arguments {
    pub configuration_path: Option<String>,
}

pub struct Application {}

impl Application {
    /// Handle current CLI arguments. When arguments cannot get parsed, it exits the program,
    /// displaying the corresponding message, and returning the proper error code.
    pub fn handle_arguments() -> Arguments {
        let matches = App::new(crate_name!())
            .version(crate_version!())
            .arg(
                Arg::new("configuration_path")
                    .short('c')
                    .long("config")
                    .takes_value(true)
                    .value_name("FILE")
                    .help("Sets the path of the configuration file")
            )
            .help_template("USAGE: {usage}\n\n{all-args}")
            .get_matches()
        ;

        Arguments {
            configuration_path: match matches.value_of("configuration_path") {
                Some(path) => Some(path.to_string()),
                None => None,
            }
        }
    }
}
