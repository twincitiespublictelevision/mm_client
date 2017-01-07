#[cfg(test)]
use mockito;

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
}

pub type Params<'a> = Vec<(&'a str, &'a str)>;

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
    pub fn new(key: &'a str, secret: &'a str) -> Client<'a> {
        Client {
            key: key,
            secret: secret,
            base: LIVE_URL,
        }
    }

    pub fn qa(key: &'a str, secret: &'a str) -> Client<'a> {
        Client {
            key: key,
            secret: secret,
            base: QA_URL,
        }
    }

    pub fn get(&self, endpoint: Endpoints, id: &str) -> CDCResult<String> {
        rq_get(vec![self.base, "/", endpoint.to_string().as_str(), "/", id, "/"]
                   .join("")
                   .as_str(),
               vec![])
    }

    pub fn list(&self, endpoint: Endpoints, params: Params) -> CDCResult<String> {
        rq_get(vec![self.base, "/", endpoint.to_string().as_str(), "/"]
                   .join("")
                   .as_str(),
               params)
    }

    pub fn url(&self, url: &str) -> CDCResult<String> {
        rq_get(url, vec![])
    }
}
