extern crate reqwest;
extern crate serde;

#[cfg(test)]
use mockito;

use reqwest::blocking::{Client as NetworkClient, RequestBuilder, Response};
use reqwest::header::CONNECTION;
use reqwest::{Method, StatusCode};
use serde::Serialize;

use std::fmt;
use std::io::Read;
use std::str;

use crate::error::MMCError;
use crate::error::MMCResult;

#[cfg(not(test))]
const LIVE_URL: &'static str = "https://media.services.pbs.org/api/v1";
#[cfg(not(test))]
const STAGING_URL: &'static str = "https://media-staging.services.pbs.org/api/v1";

#[cfg(test)]
const LIVE_URL: &'static str = mockito::SERVER_URL;
#[cfg(test)]
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

type ParentEndpoint<'a> = (Endpoints, &'a str);

#[derive(Serialize)]
struct MoveTarget {
    #[serde(skip_serializing_if = "Option::is_none")]
    show: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    season: Option<String>,
}

impl MoveTarget {
    pub fn for_endpoint(endpoint: &Endpoints, id: &str) -> MMCResult<MoveTarget> {
        match *endpoint {
            Endpoints::Season => Ok(MoveTarget {
                show: None,
                season: Some(id.to_string()),
            }),
            Endpoints::Show => Ok(MoveTarget {
                show: Some(id.to_string()),
                season: None,
            }),
            _ => Err(MMCError::UnsupportedMoveParent(endpoint.to_string())),
        }
    }
}

#[derive(Serialize)]
struct Move {
    #[serde(rename = "type")]
    _type: String,
    id: String,
    attributes: MoveTarget,
}

#[derive(Serialize)]
struct MoveRequest {
    data: Move,
}

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

impl Endpoints {
    fn singular(&self) -> String {
        match *self {
            Endpoints::Asset => "asset",
            Endpoints::Changelog => "changelog",
            Endpoints::Collection => "collection",
            Endpoints::Episode => "episode",
            Endpoints::Franchise => "franchise",
            Endpoints::Season => "season",
            Endpoints::Show => "show",
            Endpoints::Special => "special",
        }
        .to_string()
    }
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
        NetworkClient::builder()
            .build()
            .map_err(MMCError::Network)
            .and_then(|net_client| {
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
    pub fn get(&self, endpoint: Endpoints, id: &str, params: Option<Params>) -> MMCResult<String> {
        self.rq_get(
            Client::build_url(
                self.base.as_str(),
                None,
                endpoint,
                Some(id),
                params.unwrap_or(vec![]),
            )
            .as_str(),
        )
    }

    /// Attempts to fetch a list of objects from the requested Media Manager API endpoint augmented
    /// by the requested parameters
    pub fn list(&self, endpoint: Endpoints, params: Params) -> MMCResult<String> {
        self.rq_get(Client::build_url(self.base.as_str(), None, endpoint, None, params).as_str())
    }

    /// Attempts to fetch a list of child objects of the requested Media Manager API type belonging
    /// to the requested parent object augmeted by the requested parameters
    pub fn child_list(
        &self,
        endpoint: Endpoints,
        parent_id: &str,
        parent_endpoint: Endpoints,
        params: Option<Params>,
    ) -> MMCResult<String> {
        self.rq_get(
            Client::build_url(
                self.base.as_str(),
                Some((parent_endpoint, parent_id)),
                endpoint,
                None,
                params.unwrap_or(Vec::new()),
            )
            .as_str(),
        )
    }

    /// Attempts to create a new object of the provided [Endpoints](enum.Endpoints.html) for the
    /// provided parent [Endpoints](enum.Endpoints.html)
    pub fn create<T: Serialize>(
        &self,
        parent: Endpoints,
        id: &str,
        endpoint: Endpoints,
        body: &T,
    ) -> MMCResult<String> {
        self.rq_post(
            Client::build_url(
                self.base.as_str(),
                Some((parent, id)),
                endpoint,
                None,
                vec![],
            )
            .as_str(),
            body,
        )
    }

    /// Attempts to fetch the edit object specified by the [Endpoints](enum.Endpoints.html) and id
    pub fn edit(&self, endpoint: Endpoints, id: &str) -> MMCResult<String> {
        self.rq_get(
            Client::build_edit_url(self.base.as_str(), None, endpoint, Some(id), vec![]).as_str(),
        )
    }

    /// Attempts to update the object specified by the [Endpoints](enum.Endpoints.html) and id
    pub fn update<T: Serialize>(
        &self,
        endpoint: Endpoints,
        id: &str,
        body: &T,
    ) -> MMCResult<String> {
        self.rq_patch(
            Client::build_edit_url(self.base.as_str(), None, endpoint, Some(id), vec![]).as_str(),
            body,
        )
    }

    /// Attempts to delete the object specified by the [Endpoints](enum.Endpoints.html) and id
    pub fn delete(&self, endpoint: Endpoints, id: &str) -> MMCResult<String> {
        self.rq_delete(
            Client::build_edit_url(self.base.as_str(), None, endpoint, Some(id), vec![]).as_str(),
        )
    }

    /// Attempts to change the parent of an object
    pub fn change_parent(
        &self,
        parent_endpoint: Endpoints,
        parent_id: &str,
        child_endpoint: Endpoints,
        child_id: &str,
    ) -> MMCResult<String> {
        let move_request = MoveRequest {
            data: Move {
                _type: child_endpoint.singular(),
                id: child_id.to_string(),
                attributes: MoveTarget::for_endpoint(&parent_endpoint, parent_id)?,
            },
        };

        self.rq_patch(
            Client::build_url(
                self.base.as_str(),
                None,
                child_endpoint,
                Some(child_id),
                vec![],
            )
            .as_str(),
            &move_request,
        )
    }

    /// Allows for calling any arbitrary url from the Media Manager API
    pub fn url(&self, url: &str) -> MMCResult<String> {
        self.rq_get(url)
    }

    /// Shorthand for accessing a single asset
    pub fn asset(&self, id: &str, params: Option<Params>) -> MMCResult<String> {
        self.get(Endpoints::Asset, id, params)
    }

    /// Shorthand for accessing a list of assets
    pub fn assets(
        &self,
        parent_id: &str,
        parent_endpoint: Endpoints,
        params: Option<Params>,
    ) -> MMCResult<String> {
        self.child_list(Endpoints::Asset, parent_id, parent_endpoint, params)
    }

    /// Shorthand for accessing a list of changes
    pub fn changelog(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Changelog, params)
    }

    /// Shorthand for accessing a single collection
    pub fn collection(&self, id: &str, params: Option<Params>) -> MMCResult<String> {
        self.get(Endpoints::Collection, id, params)
    }

    /// Shorthand for accessing a list of collections
    pub fn collections(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Collection, params)
    }

    /// Shorthand for accessing a single episode
    pub fn episode(&self, id: &str, params: Option<Params>) -> MMCResult<String> {
        self.get(Endpoints::Episode, id, params)
    }

    /// Shorthand for accessing a list of episodes
    pub fn episodes(&self, season_id: &str, params: Option<Params>) -> MMCResult<String> {
        self.child_list(Endpoints::Episode, season_id, Endpoints::Season, params)
    }

    /// Shorthand for accessing a single franchise
    pub fn franchise(&self, id: &str, params: Option<Params>) -> MMCResult<String> {
        self.get(Endpoints::Franchise, id, params)
    }

    /// Shorthand for accessing a list of franchises
    pub fn franchises(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Franchise, params)
    }

    /// Shorthand for accessing a single season
    pub fn season(&self, id: &str, params: Option<Params>) -> MMCResult<String> {
        self.get(Endpoints::Season, id, params)
    }

    /// Shorthand for accessing a list of seasons
    pub fn seasons(&self, show_id: &str, params: Option<Params>) -> MMCResult<String> {
        self.child_list(Endpoints::Season, show_id, Endpoints::Show, params)
    }

    /// Shorthand for accessing a single special
    pub fn special(&self, id: &str, params: Option<Params>) -> MMCResult<String> {
        self.get(Endpoints::Special, id, params)
    }

    /// Shorthand for accessing a list of specials
    pub fn specials(&self, show_id: &str, params: Option<Params>) -> MMCResult<String> {
        self.child_list(Endpoints::Special, show_id, Endpoints::Show, params)
    }

    /// Shorthand for accessing a single show
    pub fn show(&self, id: &str, params: Option<Params>) -> MMCResult<String> {
        self.get(Endpoints::Show, id, params)
    }

    /// Shorthand for accessing a list of shows
    pub fn shows(&self, params: Params) -> MMCResult<String> {
        self.list(Endpoints::Show, params)
    }

    // Handle read endpoints of the API
    fn rq_get(&self, url: &str) -> MMCResult<String> {
        self.rq_send(self.client.get(url))
    }

    // Handle create endpoints of the API
    fn rq_post<T: Serialize>(&self, url: &str, body: &T) -> MMCResult<String> {
        self.rq_send(self.client.post(url).json(body))
    }

    // Handle update endpoints of the API
    fn rq_patch<T: Serialize>(&self, url: &str, body: &T) -> MMCResult<String> {
        self.rq_send(self.client.request(Method::PATCH, url).json(body))
    }

    // Handle update endpoints of the API
    fn rq_delete(&self, url: &str) -> MMCResult<String> {
        self.rq_send(self.client.request(Method::DELETE, url))
    }

    // Handle authentication and response mapping
    fn rq_send(&self, req: RequestBuilder) -> MMCResult<String> {
        req.basic_auth(self.key.to_string(), Some(self.secret.to_string()))
            .header(CONNECTION, "close")
            .send()
            .map_err(MMCError::Network)
            .and_then(Client::handle_response)
    }

    fn build_edit_url(
        base_url: &str,
        parent: Option<ParentEndpoint>,
        endpoint: Endpoints,
        id: Option<&str>,
        params: Params,
    ) -> String {
        let mut url = Client::build_url(base_url, parent, endpoint, id, params);
        url.push_str("edit/");

        url
    }

    fn build_url(
        base_url: &str,
        parent: Option<ParentEndpoint>,
        endpoint: Endpoints,
        id: Option<&str>,
        params: Params,
    ) -> String {
        // Create the new base for the returned url
        let mut url = base_url.to_string();
        url.push('/');

        // Add the parent endpoint if an endpoint and id was supplied
        if let Some(p_endpoint) = parent {
            url.push_str(p_endpoint.0.to_string().as_str());
            url.push('/');
            url.push_str(p_endpoint.1);
            url.push('/');
        }

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
            let param_string = params
                .iter()
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
        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Client::parse_success(response),
            StatusCode::BAD_REQUEST => Client::parse_bad_request(response),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(MMCError::NotAuthorized),
            StatusCode::NOT_FOUND => Err(MMCError::ResourceNotFound),
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
        r#try!(response.read_to_end(&mut buffer).map_err(MMCError::Io));

        // Generate a string from the buffer
        let result = String::from_utf8(buffer);

        // Return either successfully generated string or a conversion error
        match result {
            Ok(string) => Ok(string),
            Err(err) => Err(MMCError::Convert(err)),
        }
    }
}
