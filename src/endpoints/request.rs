extern crate serde;
extern crate serde_json;
extern crate hyper;

use self::hyper::client::Client;
use self::hyper::client::RequestBuilder;
use std::io::Read;
use self::serde_json::Value;

use error::CDCError as Error;
use error::CDCResult as Result;

pub fn rq_get(url: &str) -> Result<Value> {
    rq_make(Client::new().get(url))
}

pub fn rq_post(url: &str) -> Result<Value> {
    rq_make(Client::new().post(url))
}

pub fn rq_patch(url: &str) -> Result<Value> {
    rq_make(Client::new().patch(url))
}

fn rq_make(request: RequestBuilder) -> Result<Value> {

    // Request a response from the API endpoint and return with a
    // network error in the case of a failure
    let mut response = try!(request.send()
        .map_err(Error::Network));

    // Create a buffer to read the response stream into
    let mut buffer = Vec::new();

    // Try to read the response into the buffer and return with a
    // io error in the case of a failure
    try!(response.read_to_end(&mut buffer).map_err(Error::Io));

    // Generate a string from the buffer that will be deserialized
    let raw = String::from_utf8(buffer).unwrap();

    // Attempt to deserialize the json string
    let result = serde_json::from_str(raw.as_str());

    // Return either successfully decoded json or a Parse error
    match result {
        Ok(json) => Ok(json),
        Err(err) => Err(Error::Parse(err)),
    }
}
