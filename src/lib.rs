//! # mm_client
//!
//! The `mm_client` crate is a very small library for communicating with the PBS Media Manager API
//! easier. It provides a [Client](struct.Client.html) for querying against either the production
//! API or the staging API.
//!
//! The main goals of the crate are to:
//! * Provide authentication handling
//! * Manage API url construction
//! * Handle API error responses
//! * Make few assumptions about how responses will be used
//!
//! Currently all requests made by a [Client](struct.Client.html) are synchronous.
//!
//! # Creating a [Client](struct.Client.html)
//!
//! [Client](struct.Client.html) provides two constructors, one for the accessing the production
//! API and one for the staging API. Both constructors take an API key and secret as arguments. It
//! is recommended to create a single [Client](struct.Client.html) that is then passed around for
//! making requests.
//!
//! Note that constructing a client may fail.
//!
//! ```no_run
//! let client = Client::new("API_KEY", "API_SECRET").unwrap();
//! ```
//!
//! # Fetching a single object
//!
//! Requesting a single object can be performed by using the `get` method
//!
//! ```no_run
//! let client = Client::new("API_KEY", "API_SECRET").unwrap();
//! let response = client.get(Endpoints::Asset, "asset-id");
//! ```
//! The response string can then be handed off a JSON parser for further use.
//!
//! # Fetching a list of objects
//!
//! Requesting a list of objects can be performed by using the `list` method
//!
//! ```no_run
//! let client = Client::new("API_KEY", "API_SECRET").unwrap();
//! let params = vec![("since", "2017-02-12T00:00:00Z")];
//! let response = client.list(Endpoints::Show, params);
//! ```
//! Here a request is made for all of the show objects that have been updated since the supplied
//! date. Similar to the `get` method, the response string is available to pass to a JSON parser

#![deny(missing_docs)]
#[macro_use]
#[cfg(test)]
extern crate assert_matches;
#[cfg(test)]
extern crate mockito;
#[cfg(test)]
extern crate uuid;
#[cfg(test)]
extern crate reqwest;

mod client;
mod error;
pub use client::Client;
pub use client::Endpoints;
pub use error::MMCResult;
pub use error::MMCError;

#[cfg(test)]
mod tests {
    use client::Client;
    use client::Params;
    use client::Endpoints;
    use error::MMCResult;
    use error::MMCError;
    use mockito::mock;
    use mockito::Mock;
    use uuid::Uuid;
    use reqwest::StatusCode;

    const KEY: &'static str = "hello";
    const SECRET: &'static str = "world";
    const BASIC_AUTH: &'static str = "Basic aGVsbG86d29ybGQ=";

    fn sample_client() -> Client {
        Client::staging(KEY, SECRET).unwrap()
    }

    fn show_get(id: &str) -> MMCResult<String> {
        sample_client().get(Endpoints::Show, id)
    }

    fn show_list(params: Params) -> MMCResult<String> {
        sample_client().list(Endpoints::Show, params)
    }

    fn random_id() -> String {
        Uuid::new_v4().hyphenated().to_string()
    }

    fn mock_single(endpoint: &str, id: &str) -> Mock {
        mock("GET", vec!["/", endpoint, "/", id, "/"].join("").as_str())
    }

    fn mock_list(endpoint: &str, param_string: &str) -> Mock {
        mock("GET",
             vec!["/", endpoint, "/", param_string].join("").as_str())
    }

    #[test]
    fn handles_single_200() {
        let id = random_id();
        let mock = mock_single("shows", id.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"value\"}")
            .create_for(|| {
                let test_response = String::from("{\"name\":\"value\"}");
                assert_matches!(show_get(id.as_str()), Ok(test_response))
            })
            .remove();
    }

    #[test]
    fn handles_list_200() {
        let param_string = "?param1=value1&param2=value2";

        let mock = mock_list("shows", param_string)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"value\"}")
            .create_for(|| {
                let params = vec![("param1", "value1"), ("param2", "value2")];
                let test_response = String::from("{\"name\":\"value\"}");
                assert_matches!(show_list(params), Ok(test_response))
            })
            .remove();
    }

    #[test]
    fn handles_basic_auth() {
        let id = random_id();
        let mut param_string = "?param=".to_string();
        param_string.push_str(id.as_str());

        let mock = mock_list("shows", param_string.as_str())
            .match_header("Authorization", BASIC_AUTH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"value\"}")
            .create_for(|| {
                let params = vec![("param", id.as_str())];
                let test_response = String::from("{\"name\":\"value\"}");
                assert_matches!(show_list(params), Ok(test_response))
            })
            .remove();
    }

    #[test]
    fn handles_400() {
        let id = random_id();
        let mock = mock_single("shows", id.as_str())
            .with_status(400)
            .with_header("content-type", "text/plain")
            .with_body("Failure message from the server")
            .create_for(|| {
                let bad_rq_error = MMCError::BadRequest(String::from("Failure message from the \
                                                                      server"));

                assert_matches!(show_get(id.as_str()), Err(bad_rq_error))
            })
            .remove();
    }

    #[test]
    fn handles_401() {
        let id = random_id();
        let mock = mock_single("shows", id.as_str())
            .with_status(401)
            .create_for(|| assert_matches!(show_get(id.as_str()), Err(MMCError::NotAuthorized)))
            .remove();
    }

    #[test]
    fn handles_403() {
        let id = random_id();
        let mock = mock_single("shows", id.as_str())
            .with_status(403)
            .create_for(|| assert_matches!(show_get(id.as_str()), Err(MMCError::NotAuthorized)))
            .remove();
    }

    #[test]
    fn handles_404() {
        let id = random_id();
        let mock = mock_single("shows", id.as_str())
            .with_status(404)
            .create_for(|| assert_matches!(show_get(id.as_str()), Err(MMCError::ResourceNotFound)))
            .remove();
    }

    #[test]
    fn handles_500() {
        let id = random_id();
        let mock = mock_single("shows", id.as_str())
            .with_status(500)
            .create_for(|| {
                assert_matches!(show_get(id.as_str()),
                                Err(MMCError::APIFailure(StatusCode::InternalServerError)))
            })
            .remove();
    }

    #[test]
    fn shorthand_singles_ok() {
        let id = random_id();

        let endpoints = vec![Endpoints::Asset,
                             Endpoints::Collection,
                             Endpoints::Episode,
                             Endpoints::Franchise,
                             Endpoints::Season,
                             Endpoints::Special,
                             Endpoints::Show];

        for endpoint in endpoints.into_iter() {
            mock_single(endpoint.to_string().as_str(), id.as_str())
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("{\"name\":\"value\"}")
                .create_for(|| {
                    let test_response = String::from("{\"name\":\"value\"}");
                    assert_matches!(sample_client().get(endpoint.clone(), id.as_str()),
                                    Ok(test_response))
                })
                .remove();
        }
    }

    #[test]
    fn shorthand_list_ok() {
        let id = random_id();

        let param_string = vec!["?param1=", id.as_str(), "&param2=value2"].join("");
        let params = vec![("param1", id.as_str()), ("param2", "value2")];

        let endpoints = vec![Endpoints::Changelog,
                             Endpoints::Collection,
                             Endpoints::Franchise,
                             Endpoints::Show];

        for endpoint in endpoints.into_iter() {
            mock_list(endpoint.to_string().as_str(), param_string.as_str())
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("{\"name\":\"value\"}")
                .create_for(|| {
                    let test_response = String::from("{\"name\":\"value\"}");
                    assert_matches!(sample_client().list(endpoint.clone(), params.clone()),
                                    Ok(test_response))
                })
                .remove();
        }
    }
}
