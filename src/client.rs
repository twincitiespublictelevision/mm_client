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
const STAGING_URL: &'static str = "https://media-staging.services.pbs.org/api/v1";

#[cfg(test)]
const LIVE_URL: &'static str = mockito::SERVER_URL;
#[cfg(tet)]
const STAGING_URL: &'static str = mockito::SERVER_URL;

/// A client for communicating with the Media Manager API
#[derive(Debug)]
pub struct Client {
    key: String,
    secret: String,
    base: String,
    client: NetworkClient,
}

pub type Params<'a> = Vec<(&'a str, &'a str)>;

/// The Media Manager endpoints that are supported by [Client](struct.Client.html)
#[derive(Clone, Debug)]
pub enum Endpoints {
    /// Represents the assets endpoint
    Asset,

    /// Represents the changelog endpoint
    Changelog,

    /// Represents the collections endpoint
    Collection,

    /// Represents the episodes endpoint
    Episode,

    /// Represents the franchises endpoint
    Franchise,

    /// Represents the seasons endpoint
    Season,

    /// Represents the shows endpoint
    Show,

    /// Represents the specials endpoint
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
            "asset" | "assets" => Ok(Endpoints::Asset),
            "changelog" => Ok(Endpoints::Changelog),
            "collection" | "collections" => Ok(Endpoints::Collection),
            "episode" | "episodes" => Ok(Endpoints::Episode),
            "franchise" | "franchises" => Ok(Endpoints::Franchise),
            "season" | "seasons" => Ok(Endpoints::Season),
            "show" | "shows" => Ok(Endpoints::Show),
            "special" | "specials" => Ok(Endpoints::Special),
            x => Err(MMCError::UnknownEndpoint(x.to_string())),
        }
    }
}

impl Client {
    /// Generates a new client for the production Media Manager API
    pub fn new(key: &str, secret: &str) -> MMCResult<Client> {
        Client::client_builder(key, secret, LIVE_URL)
    }

    /// Generates a new client for the staging Media Manager API
    pub fn staging(key: &str, secret: &str) -> MMCResult<Client> {
        Client::client_builder(key, secret, STAGING_URL)
    }

    fn client_builder(key: &str, secret: &str, base: &str) -> MMCResult<Client> {
        NetworkClient::new().map_err(MMCError::Network).and_then(|net_client| {
            Ok(Client {
                key: String::from(key),
                secret: String::from(secret),
                base: String::from(base),
                client: net_client,
            })
        })
    }

    /// Attempts to fetch a single object with the requested id from the requested
    /// Media Manager API endpoint
    pub fn get(&self, endpoint: Endpoints, id: &str) -> MMCResult<String> {
        self.rq_get(Client::build_url(self.base.as_str(), endpoint, Some(id), vec![]).as_str())
    }

    /// Attempts to fetch a list of objects from the requested Media Manager API endpoint augmented
    /// by the requested parameters
    pub fn list(&self, endpoint: Endpoints, params: Params) -> MMCResult<String> {
        self.rq_get(Client::build_url(self.base.as_str(), endpoint, None, params).as_str())
    }

    /// Allows for calling any arbitrary url from the Media Manager API
    pub fn url(&self, url: &str) -> MMCResult<String> {
        self.rq_get(url)
    }

    /// Shorthand for accessing a single asset
    pub fn asset(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Episode, id)
    }

    /// Shorthand for accessing a list of changes
    pub fn changelog(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Changelog, params)
    }

    /// Shorthand for accessing a single collection
    pub fn collection(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Collection, id)
    }

    /// Shorthand for accessing a list of collections
    pub fn collections(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Collection, params)
    }

    /// Shorthand for accessing a single episode
    pub fn episode(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Episode, id)
    }

    /// Shorthand for accessing a single franchise
    pub fn franchise(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Franchise, id)
    }

    /// Shorthand for accessing a list of franchises
    pub fn franchises(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Franchise, params)
    }

    /// Shorthand for accessing a single season
    pub fn season(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Season, id)
    }

    /// Shorthand for accessing a single special
    pub fn special(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Special, id)
    }

    /// Shorthand for accessing a single show
    pub fn show(&self, id: &str) -> MMCResult<String> {
        self.get(Endpoints::Show, id)
    }

    /// Shorthand for accessing a list of shows
    pub fn shows(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Show, params)
    }

    fn rq_get(&self, url: &str) -> MMCResult<String> {
        self.client
            .get(url)
            .header(Authorization(Basic {
                username: self.key.to_string(),
                password: Some(self.secret.to_string()),
            }))
            .header(Connection::close())
            .send()
            .map_err(MMCError::Network)
            .and_then(Client::handle_response)
    }

    fn build_url(base_url: &str, endpoint: Endpoints, id: Option<&str>, params: Params) -> String {

        // Create the new base for the returned url
        let mut url = base_url.to_string();
        url.push('/');

        // Parse the requested endpoint
        let endpoint_string = endpoint.to_string();
        url.push_str(endpoint_string.as_str());
        url.push('/');

        // Optional add the id if it was supplied
        if let Some(id_val) = id {
            url.push_str(id_val);
            url.push('/');
        }

        // Add the query parameters to the url
        url + Client::format_params(params).as_str()
    }

    fn format_params(params: Params) -> String {
        if !params.is_empty() {
            let param_string = params.iter()
                .map(|&(name, value)| format!("{}={}", name, value))
                .collect::<Vec<String>>()
                .join("&");

            let mut args = "?".to_owned();
            args.push_str(param_string.as_str());
            args
        } else {
            String::new()
        }
    }

    fn handle_response(response: Response) -> MMCResult<String> {
        match *response.status() {
            StatusCode::Ok => Client::parse_success(response),
            StatusCode::BadRequest => Client::parse_bad_request(response),
            StatusCode::Unauthorized | StatusCode::Forbidden => Err(MMCError::NotAuthorized),
            StatusCode::NotFound => Err(MMCError::ResourceNotFound),
            x => Err(MMCError::APIFailure(x)),
        }
    }

    fn parse_success(response: Response) -> MMCResult<String> {
        Client::parse_response_body(response)
    }

    fn parse_bad_request(response: Response) -> MMCResult<String> {
        Client::parse_response_body(response).and_then(|body| Err(MMCError::BadRequest(body)))
    }

    fn parse_response_body(mut response: Response) -> MMCResult<String> {

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
