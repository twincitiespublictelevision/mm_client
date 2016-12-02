mod endpoints;
mod client;
mod error;
pub use client::Client;
pub use error::CDCResult;
pub use error::CDCError;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use client::Client;
        let client = Client::qa("", "");
        let show1 = client.shows(2);
        let show2 = client.shows(3);
    }
}
