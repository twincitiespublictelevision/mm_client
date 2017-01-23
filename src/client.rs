extern crate reqwest;

#[cfg(test)]
use mockito;

use self::reqwest::Client as NetworkClient;

use error::CDCError;
use error::CDCResult;
use request::rq_get;
use std::fmt;

#[cfg(not(test))]
const LIVE_URL: &'static str = "https://media-qa.services.pbs.org/api/v1";
#[cfg(not(test))]
const QA_URL: &'static str = "https://media-qa.services.pbs.org/api/v1";

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
        client_builder(key, secret, LIVE_URL)
    }

    pub fn qa(key: &'a str, secret: &'a str) -> CDCResult<Client<'a>> {
        client_builder(key, secret, QA_URL)
    }

    pub fn get(&self, endpoint: Endpoints, id: &str) -> CDCResult<String> {
        rq_get(&self.client,
               vec![self.base, "/", endpoint.to_string().as_str(), "/", id, "/"]
                   .join("")
                   .as_str(),
               vec![])
    }

    pub fn list(&self, endpoint: Endpoints, params: Params) -> CDCResult<String> {
        rq_get(&self.client,
               vec![self.base, "/", endpoint.to_string().as_str(), "/"]
                   .join("")
                   .as_str(),
               params)
    }

    pub fn url(&self, url: &str) -> CDCResult<String> {
        rq_get(&self.client, url, vec![])
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
}

fn client_builder<'a>(key: &'a str, secret: &'a str, base: &'a str) -> CDCResult<Client<'a>> {
    NetworkClient::new().map_err(CDCError::Network).and_then(|netClient| {
        Ok(Client {
            key: key,
            secret: secret,
            base: base,
            client: netClient,
        })
    })
}
