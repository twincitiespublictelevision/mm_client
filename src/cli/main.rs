//! # mm_cli
//!
//! The `mm_cli` feature provides the ability to compile a very minimal cli for querying against
//! the PBS Media Manager API. It is built directly on top of the [Client](struct.Client.html)
//! and allows for querying against either the production API or the staging API.
//!
//! ### Configuration
//!
//! Configuration for the cli is contained within a `config.toml` file. It uses a `live` key to
//! designate your credentials for the production API and a `staging` key for the staging API.
//!
//! The confiuration file can be created manually or the cli can generate it with some assistance.
//! If you run the cli without a configuration file then it will attempt to generate one and prompt
//! for the necessary credentials. Settings can be rewritten by running the cli with the `init`
//! flag.

#![deny(missing_docs)]
extern crate app_dirs;
extern crate clap;
extern crate mm_client;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod error;
mod config;

use app_dirs::{AppDataType, AppInfo, get_app_dir};
use clap::{App, Arg};
use mm_client::Client;
use mm_client::Endpoints;
use mm_client::MMCResult;

use std::str::FromStr;

use error::CLIError;
use config::Config;

fn main() {

    let matches = App::new("MediaManager CLI")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("init")
            .long("init")
            .takes_value(false)
            .help("Creates/replaces a config.toml file"))
        .arg(Arg::with_name("type")
            .takes_value(true)
            .required(false)
            .help("Object type to query for"))
        .arg(Arg::with_name("id")
            .takes_value(true)
            .required(false)
            .help("Object id to query for"))
        .arg(Arg::with_name("staging")
            .short("s")
            .long("staging")
            .takes_value(false)
            .help("Runs query against the staging environment"))
        .get_matches();

    let info = AppInfo {
        name: env!("CARGO_PKG_NAME"),
        author: "tpt",
    };

    let config_path = get_app_dir(AppDataType::UserConfig, &info, "/")
        .and_then(|mut path| {
            path.push("config.toml");
            Ok(path)
        })
        .expect("Failed to run. Unable to determine default config location.");

    let path = config_path.to_str().expect("Failed to parse config location.");

    if matches.is_present("init") {
        Config::create(path);
    };

    match (matches.value_of("type"), matches.value_of("id")) {
        (Some(endpoint), Some(id)) => {

            Config::parse_config(path).and_then(|config| {
                let result = Endpoints::from_str(endpoint)
                    .or(Err(CLIError::Endpoint))
                    .and_then(|ep| rq_endpoint(&config, matches.is_present("staging"), ep, id));

                // Handle the result from the client, outputting it to the user
                match result {
                    Ok(ref value) => println!("{}", value),
                    Err(ref error) => println!("An error occured: {}", error),
                };

                result
            });
        }
        _ => (),
    }
}

fn rq_endpoint(config: &Config,
               is_staging: bool,
               endpoint: Endpoints,
               id: &str)
               -> Result<String, CLIError> {
    let conf = if is_staging {
        &config.staging
    } else {
        &config.live
    };

    // Attempt to get a response from the requested endpoint if it is
    // one of the available endpoints
    match *conf {
        Some(ref key_sec) => {

            let client = if is_staging {
                Client::staging(key_sec.key.as_str(), key_sec.secret.as_str())
            } else {
                Client::new(key_sec.key.as_str(), key_sec.secret.as_str())
            };

            client.map_err(CLIError::Network)
                .and_then(|cl| handle_client_response(cl.get(endpoint, id)))
        }
        None => Err(CLIError::EndpointConfigMissing),
    }
}

/// Handles responses from the Core Data Client and transforms them into
/// a Result that is ready for output to a user
fn handle_client_response(result: MMCResult<String>) -> Result<String, CLIError> {
    match result {
        Ok(json_string) => to_pretty_print(json_string),
        Err(err) => Err(CLIError::Network(err)),
    }
}

// Accepts a String of json, and returns the Result of attempting to transform
// it into a pretty printable String
fn to_pretty_print(json_string: String) -> Result<String, CLIError> {

    // Attempt to deserialize the json string
    let parsed: Result<serde_json::Value, serde_json::Error> =
        serde_json::from_str(json_string.as_str());

    // Return either successfully decoded json or a Format error
    match parsed {
        Ok(json) => serde_json::ser::to_string_pretty(&json).map_err(CLIError::Format),
        Err(err) => Err(CLIError::Format(err)),
    }
}
