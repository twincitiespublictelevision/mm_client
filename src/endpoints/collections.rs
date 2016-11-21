extern crate serde_json;

use self::serde_json::Value;

use error::CDCResult as Result;

use endpoints::request::rq_get;

const SLUG: &'static str = "collections";

pub struct CollectionEndpoint<'a> {
    key: &'a str,
    secret: &'a str,
    base: &'a str,
}

impl<'a> CollectionEndpoint<'a> {
    pub fn new(key: &'a str, secret: &'a str, base: &'a str) -> CollectionEndpoint<'a> {
        CollectionEndpoint {
            key: key,
            secret: secret,
            base: base,
        }
    }

    pub fn get(&self, id: &str) -> Result<Value> {

        rq_get(vec![self.base, SLUG, "/", id, "/"]
            .join("")
            .as_str())
    }

    pub fn list(&self) -> Result<Value> {

        rq_get(vec![self.base, SLUG, "/"]
            .join("")
            .as_str())
    }
}
