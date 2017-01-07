extern crate hyper;

use std::error::Error;
use std::result::Result;
use std::fmt;
use std::io;
use std::string;

// Result type that the client can return
pub type CDCResult<T> = Result<T, CDCError>;

#[derive(Debug)]
pub enum CDCError {
    // Generated by when a request tries to access an resource it is not authorized for
    NotAuthorized,

    // Generated by a failure to find a requested resource
    ResourceNotFound,

    // Generated by unknown server failures from the remote server
    APIFailure,

    // Generated by a bad request response from the server with a reason attached
    BadRequest(String),

    // Generated by a failure to parse an API response
    Convert(string::FromUtf8Error),

    // Generated by the networking client
    Network(hyper::error::Error),

    // Generated by handling of network responses
    Io(io::Error),
}

impl fmt::Display for CDCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CDCError::NotAuthorized => {
                write!(f,
                       "Supplied credentials are not authorized to access this resource")
            }
            CDCError::ResourceNotFound => write!(f, "Specified resource could not be found"),
            CDCError::APIFailure => write!(f, "Unknown failure of the API endpoint"),
            CDCError::BadRequest(ref err_msg) => write!(f, "API did not understand request"),
            CDCError::Convert(ref err) => err.fmt(f),
            CDCError::Network(ref err) => err.fmt(f),
            CDCError::Io(ref err) => err.fmt(f),
        }
    }
}

impl Error for CDCError {
    fn description(&self) -> &str {
        match *self {
            CDCError::NotAuthorized => "",
            CDCError::ResourceNotFound => "",
            CDCError::APIFailure => "",
            CDCError::BadRequest(ref err_msg) => "",
            CDCError::Convert(ref err) => err.description(),
            CDCError::Network(ref err) => err.description(),
            CDCError::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            CDCError::NotAuthorized => None,
            CDCError::ResourceNotFound => None,
            CDCError::APIFailure => None,
            CDCError::BadRequest(ref err_msg) => None,
            CDCError::Convert(ref err) => Some(err),
            CDCError::Network(ref err) => Some(err),
            CDCError::Io(ref err) => Some(err),
        }
    }
}

impl From<string::FromUtf8Error> for CDCError {
    fn from(err: string::FromUtf8Error) -> CDCError {
        CDCError::Convert(err)
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
