use endpoints::ShowEndpoint;

pub struct Client {
    key: String,
    secret: String,
}

impl Client {
    pub fn new(key: String, secret: String) -> Client {
        Client {
            key: key,
            secret: secret,
        }
    }

    pub fn shows(&self) -> ShowEndpoint {
        ShowEndpoint::new(self.key.clone(), self.secret.clone())
    }
}
