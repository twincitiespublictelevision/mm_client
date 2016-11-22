use error::CDCResult as Result;

use endpoints::request::rq_get;

const SLUG: &'static str = "shows";

pub struct ShowEndpoint<'a> {
    key: &'a str,
    secret: &'a str,
    base: &'a str,
}

impl<'a> ShowEndpoint<'a> {
    pub fn new(key: &'a str, secret: &'a str, base: &'a str) -> ShowEndpoint<'a> {
        ShowEndpoint {
            key: key,
            secret: secret,
            base: base,
        }
    }

    pub fn get(&self, id: &str) -> Result<String> {

        rq_get(vec![self.base, SLUG, "/", id, "/"]
            .join("")
            .as_str())
    }

    pub fn list(&self) -> Result<String> {

        rq_get(vec![self.base, SLUG, "/"]
            .join("")
            .as_str())
    }
}
