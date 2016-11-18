extern crate hyper;
extern crate serde_json;

use std::error::Error;
use std::result::Result;
use std::fmt;
use std::io;

// Result type that the client can return
pub type CDCResult<T> = Result<T, CDCError>;

#[derive(Debug)]
pub enum CDCError {
    // Generated by a failure to parse an API response
    Parse(serde_json::error::Error),

    // Generated by the networking client
    Network(hyper::error::Error),

    // Generated by handling of network responses
    Io(io::Error),
}

impl fmt::Display for CDCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CDCError::Parse(ref err) => err.fmt(f),
            CDCError::Network(ref err) => err.fmt(f),
            CDCError::Io(ref err) => err.fmt(f),
        }
    }
}

impl Error for CDCError {
    fn description(&self) -> &str {
        match *self {
            CDCError::Parse(ref err) => err.description(),
            CDCError::Network(ref err) => err.description(),
            CDCError::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            CDCError::Parse(ref err) => Some(err),
            CDCError::Network(ref err) => Some(err),
            CDCError::Io(ref err) => Some(err),
        }
    }
}

impl From<serde_json::error::Error> for CDCError {
    fn from(err: serde_json::error::Error) -> CDCError {
        CDCError::Parse(err)
    }
}

impl From<hyper::error::Error> for CDCError {
    fn from(err: hyper::error::Error) -> CDCError {
        CDCError::Network(err)
    }
}

impl From<io::Error> for CDCError {
    fn from(err: io::Error) -> CDCError {
        CDCError::Io(err)
    }
}
