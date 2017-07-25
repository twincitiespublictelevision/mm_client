extern crate serde_json;
extern crate mm_client;

use std::error::Error;
use std::io;
use std::fmt;

use self::mm_client::MMCError;

#[derive(Debug)]
pub enum CLIError {
    InvalidConfig,
    EndpointConfigMissing,
    Endpoint,
    ConfigStorageFailure(io::Error),
    Format(serde_json::error::Error),
    Network(MMCError),
}

impl fmt::Display for CLIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CLIError::InvalidConfig => {
                write!(
                    f,
                    "Supplied config.toml could not be understood. Try checking for a \
                        misspelled or missing property."
                )
            }
            CLIError::EndpointConfigMissing => {
                write!(
                    f,
                    "config.toml is missing the key/secret pair for this endpoint."
                )
            }
            CLIError::Endpoint => {
                write!(
                    f,
                    "Requested endpoint is not in the list of known endpoints."
                )
            }
            CLIError::ConfigStorageFailure(ref err) => err.fmt(f),
            CLIError::Format(_) => write!(f, "Failure to format response."),
            CLIError::Network(ref err) => err.fmt(f),
        }
    }
}

impl Error for CLIError {
    fn description(&self) -> &str {
        match *self {
            CLIError::InvalidConfig => {
                "Supplied config.toml could not be understood. Try checking for a misspelled or \
                 missing property."
            }
            CLIError::EndpointConfigMissing => {
                "config.toml is missing the key/secret pair for this endpoint/"
            }
            CLIError::Endpoint => "Requested endpoint is not in the list of known endpoints.",
            CLIError::ConfigStorageFailure(ref err) => err.description(),
            CLIError::Format(_) => "Unable to format the response from the server.",
            CLIError::Network(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            CLIError::InvalidConfig => None,
            CLIError::EndpointConfigMissing => None,
            CLIError::Endpoint => None,
            CLIError::ConfigStorageFailure(ref err) => Some(err),
            CLIError::Format(ref err) => Some(err),
            CLIError::Network(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for CLIError {
    fn from(err: io::Error) -> CLIError {
        CLIError::ConfigStorageFailure(err)
    }
}

impl From<serde_json::error::Error> for CLIError {
    fn from(err: serde_json::error::Error) -> CLIError {
        CLIError::Format(err)
    }
}

impl From<mm_client::MMCError> for CLIError {
    fn from(err: mm_client::MMCError) -> CLIError {
        CLIError::Network(err)
    }
}
