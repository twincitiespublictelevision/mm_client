extern crate hyper;

use self::hyper::client::Client;
use self::hyper::client::RequestBuilder;
use std::io::Read;

use error::CDCError as Error;
use error::CDCResult as Result;

pub fn rq_get(url: &str) -> Result<String> {
    rq_make(Client::new().get(url))
}

pub fn rq_post(url: &str) -> Result<String> {
    rq_make(Client::new().post(url))
}

pub fn rq_patch(url: &str) -> Result<String> {
    rq_make(Client::new().patch(url))
}

fn rq_make(request: RequestBuilder) -> Result<String> {

    // Request a response from the API endpoint and return with a
    // network error in the case of a failure
    let mut response = try!(request.send()
        .map_err(Error::Network));

    // Create a buffer to read the response stream into
    let mut buffer = Vec::new();

    // Try to read the response into the buffer and return with a
    // io error in the case of a failure
    try!(response.read_to_end(&mut buffer).map_err(Error::Io));

    // Generate a string from the buffer
    let result = String::from_utf8(buffer);

    // Return either successfully generated string or a conversion error
    match result {
        Ok(string) => Ok(string),
        Err(err) => Err(Error::Convert(err)),
    }
}
