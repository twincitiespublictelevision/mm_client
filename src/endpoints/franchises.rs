use error::CDCResult as Result;

use endpoints::request::rq_get;

const SLUG: &'static str = "franchises";

pub struct FranchiseEndpoint<'a> {
    key: &'a str,
    secret: &'a str,
    base: &'a str,
}

impl<'a> FranchiseEndpoint<'a> {
    pub fn new(key: &'a str, secret: &'a str, base: &'a str) -> FranchiseEndpoint<'a> {
        FranchiseEndpoint {
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
