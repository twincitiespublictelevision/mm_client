extern crate reqwest;

#[cfg(test)]
use mockito;

use self::reqwest::Client as NetworkClient;
use self::reqwest::header::{Authorization, Basic, Connection};
use self::reqwest::Response;
use self::reqwest::StatusCode;

use std::fmt;
use std::io::Read;
use std::str;

use error::MMCError;
use error::MMCResult;

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

impl str::FromStr for Endpoints {
    type Err = MMCError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asset" => Ok(Endpoints::Asset),
            "assets" => Ok(Endpoints::Asset),
            "changelog" => Ok(Endpoints::Changelog),
            "collection" => Ok(Endpoints::Collection),
            "collections" => Ok(Endpoints::Collection),
            "episode" => Ok(Endpoints::Episode),
            "episodes" => Ok(Endpoints::Episode),
            "franchise" => Ok(Endpoints::Franchise),
            "franchises" => Ok(Endpoints::Franchise),
            "season" => Ok(Endpoints::Season),
            "seasons" => Ok(Endpoints::Season),
            "show" => Ok(Endpoints::Show),
            "shows" => Ok(Endpoints::Show),
            "special" => Ok(Endpoints::Special),
            "specials" => Ok(Endpoints::Special),
            x => Err(MMCError::UnknownEndpoint(x.to_string())),
        }
    }
}

impl<'a> Client<'a> {
    pub fn new(key: &'a str, secret: &'a str) -> MMCResult<Client<'a>> {
        Client::client_builder(key, secret, LIVE_URL)
    }

    pub fn qa(key: &'a str, secret: &'a str) -> MMCResult<Client<'a>> {
        Client::client_builder(key, secret, QA_URL)
    }

    fn client_builder(key: &'a str, secret: &'a str, base: &'a str) -> MMCResult<Client<'a>> {
        NetworkClient::new().map_err(MMCError::Network).and_then(|net_client| {
            Ok(Client {
                key: key,
                secret: secret,
                base: base,
                client: net_client,
            })
        })
    }

    pub fn get(&self, endpoint: Endpoints, id: &str) -> MMCResult<String> {
        self.rq_get(vec![self.base, "/", endpoint.to_string().as_str(), "/", id, "/"]
                        .join("")
                        .as_str(),
                    vec![])
    }

    pub fn list(&self, endpoint: Endpoints, params: Params) -> MMCResult<String> {
        self.rq_get(vec![self.base, "/", endpoint.to_string().as_str(), "/"]
                        .join("")
                        .as_str(),
                    params)
    }

    pub fn url(&self, url: &str) -> MMCResult<String> {
        self.rq_get(url, vec![])
    }

    pub fn asset(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Episode, id)
    }

    pub fn changelog(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Changelog, params)
    }

    pub fn collection(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Collection, id)
    }

    pub fn collections(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Collection, params)
    }

    pub fn episode(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Episode, id)
    }

    pub fn franchise(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Franchise, id)
    }

    pub fn franchises(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Franchise, params)
    }

    pub fn season(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Season, id)
    }

    pub fn special(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Special, id)
    }

    pub fn show(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Show, id)
    }

    pub fn shows(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Show, params)
    }

    fn rq_get(&self, base_url: &str, params: Params) -> MMCResult<String> {
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
            .map_err(MMCError::Network)
            .and_then(|response| self.handle_response(response))
    }

    fn handle_response(&self, response: Response) -> MMCResult<String> {
        match *response.status() {
            StatusCode::Ok => self.parse_success(response),
            StatusCode::BadRequest => self.parse_bad_request(response),
            StatusCode::Unauthorized => Err(MMCError::NotAuthorized),
            StatusCode::Forbidden => Err(MMCError::NotAuthorized),
            StatusCode::NotFound => Err(MMCError::ResourceNotFound),
            x => Err(MMCError::APIFailure(x)),
        }
    }

    fn parse_success(&self, response: Response) -> MMCResult<String> {
        self.parse_response_body(response)
    }

    fn parse_bad_request(&self, response: Response) -> MMCResult<String> {
        self.parse_response_body(response).and_then(|body| Err(MMCError::BadRequest(body)))
    }

    fn parse_response_body(&self, mut response: Response) -> MMCResult<String> {

        // Create a buffer to read the response stream into
        let mut buffer = Vec::new();

        // Try to read the response into the buffer and return with a
        // io error in the case of a failure
        try!(response.read_to_end(&mut buffer).map_err(MMCError::Io));

        // Generate a string from the buffer
        let result = String::from_utf8(buffer);

        // Return either successfully generated string or a conversion error
        match result {
            Ok(string) => Ok(string),
            Err(err) => Err(MMCError::Convert(err)),
        }
    }
}
