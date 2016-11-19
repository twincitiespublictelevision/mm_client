use endpoints::AssetEndpoint;
use endpoints::CollectionEndpoint;
use endpoints::EpisodeEndpoint;
use endpoints::FranchiseEndpoint;
use endpoints::SeasonEndpoint;
use endpoints::ShowEndpoint;
use endpoints::SpecialEndpoint;

pub struct Client<'a> {
    key: &'a str,
    secret: &'a str,
    base: &'a str,
}

impl<'a> Client<'a> {
    pub fn new(key: &'a str, secret: &'a str) -> Client<'a> {
        Client {
            key: key,
            secret: secret,
            base: "https://media-qa.services.pbs.org/api/v1/",
        }
    }

    pub fn qa(key: &'a str, secret: &'a str) -> Client<'a> {
        Client {
            key: key,
            secret: secret,
            base: "https://media-qa.services.pbs.org/api/v1/",
        }
    }

    pub fn assets(&self) -> AssetEndpoint {
        AssetEndpoint::new(self.key, self.secret, self.base)
    }

    pub fn collections(&self) -> CollectionEndpoint {
        CollectionEndpoint::new(self.key, self.secret, self.base)
    }

    pub fn episodes(&self) -> EpisodeEndpoint {
        EpisodeEndpoint::new(self.key, self.secret, self.base)
    }

    pub fn franchises(&self) -> FranchiseEndpoint {
        FranchiseEndpoint::new(self.key, self.secret, self.base)
    }

    pub fn seasons(&self) -> SeasonEndpoint {
        SeasonEndpoint::new(self.key, self.secret, self.base)
    }

    pub fn shows(&self) -> ShowEndpoint {
        ShowEndpoint::new(self.key, self.secret, self.base)
    }

    pub fn specials(&self) -> SpecialEndpoint {
        SpecialEndpoint::new(self.key, self.secret, self.base)
    }
}
