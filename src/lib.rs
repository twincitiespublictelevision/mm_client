mod error;
mod endpoints;
pub mod client;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use client::Client;
        let client = Client::qa("", "");
        let show1 = client.shows();
        let show2 = client.shows();
    }
}
