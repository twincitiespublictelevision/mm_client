extern crate reqwest;

#[cfg(test)]
use mockito;

use self::reqwest::Client as NetworkClient;
use self::reqwest::header::{Authorization, Basic, Connection};
use self::reqwest::Response;
use self::reqwest::StatusCode;

use std::fmt;
use std::io::Read;

use error::CDCError;
use error::CDCResult;

#[cfg(not(test))]
const LIVE_URL: &'static str = "https://media.services.pbs.org/api/v1";
#[cfg(not(test))]
const QA_URL: &'static str = "https://media-staging.services.pbs.org/api/v1";

#[cfg(test)]
const LIVE_URL: &'static str = mockito::SERVER_URL;
#[cfg(test)]
const QA_URL: &'static str = mockito::SERVER_URL;

pub struct Client<'a> {
    key: &'a str,
    secret: &'a str,
    base: &'a str,
    client: NetworkClient,
}

pub type Params<'a> = Vec<(&'a str, &'a str)>;

#[derive(Debug)]
pub enum Endpoints {
    Asset,
    Changelog,
    Collection,
    Episode,
    Franchise,
    Season,
    Show,
    Special,
}

impl fmt::Display for Endpoints {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_form = match *self {
            Endpoints::Asset => "assets",
            Endpoints::Changelog => "changelog",
            Endpoints::Collection => "collections",
            Endpoints::Episode => "episodes",
            Endpoints::Franchise => "franchises",
            Endpoints::Season => "seasons",
            Endpoints::Show => "shows",
            Endpoints::Special => "specials",
        };

        write!(f, "{}", string_form)
    }
}

impl<'a> Client<'a> {
    pub fn new(key: &'a str, secret: &'a str) -> CDCResult<Client<'a>> {
        Client::client_builder(key, secret, LIVE_URL)
    }

    pub fn qa(key: &'a str, secret: &'a str) -> CDCResult<Client<'a>> {
        Client::client_builder(key, secret, QA_URL)
    }

    fn client_builder(key: &'a str, secret: &'a str, base: &'a str) -> CDCResult<Client<'a>> {
        NetworkClient::new().map_err(CDCError::Network).and_then(|net_client| {
            Ok(Client {
                key: key,
                secret: secret,
                base: base,
                client: net_client,
            })
        })
    }

    pub fn get(&self, endpoint: Endpoints, id: &str) -> CDCResult<String> {
        self.rq_get(vec![self.base, "/", endpoint.to_string().as_str(), "/", id, "/"]
                        .join("")
                        .as_str(),
                    vec![])
    }

    pub fn list(&self, endpoint: Endpoints, params: Params) -> CDCResult<String> {
        self.rq_get(vec![self.base, "/", endpoint.to_string().as_str(), "/"]
                        .join("")
                        .as_str(),
                    params)
    }

    pub fn url(&self, url: &str) -> CDCResult<String> {
        self.rq_get(url, vec![])
    }

    pub fn asset(&self, id: &str) -> CDCResult<String> {
        self.get(Endpoints::Episode, id)
    }

    pub fn changelog(&self, params: Params) -> CDCResult<String> {
        self.list(Endpoints::Changelog, params)
    }

    pub fn collection(&self, id: &str) -> CDCResult<String> {
        self.get(Endpoints::Collection, id)
    }

    pub fn collections(&self, params: Params) -> CDCResult<String> {
        self.list(Endpoints::Collection, params)
    }

    pub fn episode(&self, id: &str) -> CDCResult<String> {
        self.get(Endpoints::Episode, id)
    }

    pub fn franchise(&self, id: &str) -> CDCResult<String> {
        self.get(Endpoints::Franchise, id)
    }

    pub fn franchises(&self, params: Params) -> CDCResult<String> {
        self.list(Endpoints::Franchise, params)
    }

    pub fn season(&self, id: &str) -> CDCResult<String> {
        self.get(Endpoints::Season, id)
    }

    pub fn special(&self, id: &str) -> CDCResult<String> {
        self.get(Endpoints::Special, id)
    }

    pub fn show(&self, id: &str) -> CDCResult<String> {
        self.get(Endpoints::Show, id)
    }

    pub fn shows(&self, params: Params) -> CDCResult<String> {
        self.list(Endpoints::Show, params)
    }

    fn rq_get(&self, base_url: &str, params: Params) -> CDCResult<String> {
        let mut param_string = params.iter()
            .map(|&(name, value)| format!("{}={}", name, value))
            .collect::<Vec<String>>()
            .join("&");

        if !params.is_empty() {
            param_string = "?".to_owned() + param_string.as_str();
        }

        self.client
            .get(format!("{}{}", base_url, param_string).as_str())
            .header(Authorization(Basic {
                username: self.key.to_string(),
                password: Some(self.secret.to_string()),
            }))
            .header(Connection::close())
            .send()
            .map_err(CDCError::Network)
            .and_then(|response| self.handle_response(response))
    }

    fn handle_response(&self, response: Response) -> CDCResult<String> {
        match *response.status() {
            StatusCode::Ok => self.parse_success(response),
            StatusCode::BadRequest => self.parse_bad_request(response),
            StatusCode::Unauthorized => Err(CDCError::NotAuthorized),
            StatusCode::Forbidden => Err(CDCError::NotAuthorized),
            StatusCode::NotFound => Err(CDCError::ResourceNotFound),
            _ => Err(CDCError::APIFailure),
        }
    }

    fn parse_success(&self, response: Response) -> CDCResult<String> {
        self.parse_response_body(response)
    }

    fn parse_bad_request(&self, response: Response) -> CDCResult<String> {
        self.parse_response_body(response).and_then(|body| Err(CDCError::BadRequest(body)))
    }

    fn parse_response_body(&self, mut response: Response) -> CDCResult<String> {

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
}
