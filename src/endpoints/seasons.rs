extern crate serde_json;

use self::serde_json::Value;

use error::CDCResult as Result;

use endpoints::request::rq_get;

const SLUG: &'static str = "seasons";

pub struct SeasonEndpoint<'a> {
    key: &'a str,
    secret: &'a str,
    base: &'a str,
}

impl<'a> SeasonEndpoint<'a> {
    pub fn new(key: &'a str, secret: &'a str, base: &'a str) -> SeasonEndpoint<'a> {
        SeasonEndpoint {
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
}
