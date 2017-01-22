extern crate reqwest;

use self::reqwest::Client;
use self::reqwest::Response;
use self::reqwest::StatusCode;
use std::io::Read;

use error::CDCError;
use error::CDCResult;
use client::Params;

pub fn rq_get(client: &Client, base_url: &str, params: Params) -> CDCResult<String> {
    let mut param_string = params.iter()
        .map(|&(name, value)| format!("{}={}", name, value))
        .collect::<Vec<String>>()
        .join("&");

    if !params.is_empty() {
        param_string = "?".to_owned() + param_string.as_str();
    }

    client.get(format!("{}{}", base_url, param_string).as_str())
        .send()
        .map_err(CDCError::Network)
        .and_then(|response| handle_response(response))
}

fn handle_response(response: Response) -> CDCResult<String> {
    match *response.status() {
        StatusCode::Ok => parse_success(response),
        StatusCode::BadRequest => parse_bad_request(response),
        StatusCode::Unauthorized => Err(CDCError::NotAuthorized),
        StatusCode::Forbidden => Err(CDCError::NotAuthorized),
        StatusCode::NotFound => Err(CDCError::ResourceNotFound),
        _ => Err(CDCError::APIFailure),
    }
}

fn parse_success(response: Response) -> CDCResult<String> {
    parse_response_body(response)
}

fn parse_bad_request(response: Response) -> CDCResult<String> {
    parse_response_body(response).and_then(|body| Err(CDCError::BadRequest(body)))
}

fn parse_response_body(mut response: Response) -> CDCResult<String> {

    // Create a buffer to read the response stream into
    let mut buffer = Vec::new();

    // Try to read the response into the buffer and return with a
    // io error in the case of a failure
    try!(response.read_to_end(&mut buffer).map_err(CDCError::Io));

    // Generate a string from the buffer
    let result = String::from_utf8(buffer);

    // Return either successfully generated string or a conversion error
    match result {
        Ok(string) => Ok(string),
        Err(err) => Err(CDCError::Convert(err)),
    }
}
