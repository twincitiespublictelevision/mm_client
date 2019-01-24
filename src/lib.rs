//! # mm_client
//!
//! The `mm_client` crate is a very small library for communicating with the PBS Media Manager API
//! easier. It provides a [Client](struct.Client.html) for querying against either the production
//! API or the staging API.
//!
//! The main goals of the crate are to:
//!
//!  * Provide authentication handling
//!  * Manage API url construction
//!  * Handle API error responses
//!  * Make few assumptions about how responses will be used
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
//! use mm_client::Client;
//!
//! let client = Client::new("API_KEY", "API_SECRET").unwrap();
//! ```
//!
//! # Fetching a single object
//!
//! Requesting a single object can be performed by using the `get` method
//!
//! ```no_run
//! use mm_client::Client;
//! use mm_client::Endpoints;
//!
//! let client = Client::new("API_KEY", "API_SECRET").unwrap();
//! let response = client.get(Endpoints::Asset, "asset-id", None);
//! ```
//! The response string can then be handed off a JSON parser for further use.
//!
//! # Fetching a list of objects
//!
//! Requesting a list of objects can be performed by using the `list` method
//!
//! ```no_run
//! use mm_client::Client;
//! use mm_client::Endpoints;
//!
//! let client = Client::new("API_KEY", "API_SECRET").unwrap();
//! let params = vec![("since", "2017-02-12T00:00:00Z")];
//! let response = client.list(Endpoints::Show, params);
//! ```
//! Here a request is made for all of the show objects that have been updated since the supplied
//! date. Similar to the `get` method, the response string is available to pass to a JSON parser

#![deny(missing_docs)]
#[cfg(test)]
extern crate mockito;
#[cfg(test)]
extern crate reqwest;
#[cfg(test)]
extern crate serde;
#[cfg(test)]
extern crate serde_json;
#[cfg(test)]
extern crate uuid;

mod client;
mod error;
pub use crate::client::Client;
pub use crate::client::Endpoints;
pub use crate::error::MMCError;
pub use crate::error::MMCResult;

#[cfg(test)]
mod tests {
    use mockito::mock;
    use mockito::Mock;
    use reqwest::StatusCode;
    use serde::Serialize;
    use uuid::Uuid;

    use crate::client::Client;
    use crate::client::Endpoints;
    use crate::client::Params;
    use crate::error::MMCError;
    use crate::error::MMCResult;

    const KEY: &'static str = "hello";
    const SECRET: &'static str = "world";
    const BASIC_AUTH: &'static str = "Basic aGVsbG86d29ybGQ=";

    #[derive(Serialize)]
    struct EmptyReq {}

    fn sample_client() -> Client {
        Client::staging(KEY, SECRET).unwrap()
    }

    fn show_get(id: &str, params: Option<Params>) -> MMCResult<String> {
        sample_client().get(Endpoints::Show, id, params)
    }

    fn show_list(params: Params) -> MMCResult<String> {
        sample_client().list(Endpoints::Show, params)
    }

    fn show_create<T: Serialize>(id: &str, body: &T) -> MMCResult<String> {
        sample_client().create(Endpoints::Show, id, Endpoints::Asset, body)
    }

    fn show_edit(id: &str) -> MMCResult<String> {
        sample_client().edit(Endpoints::Asset, id)
    }

    fn show_update<T: Serialize>(id: &str, body: &T) -> MMCResult<String> {
        sample_client().update(Endpoints::Asset, id, body)
    }

    fn show_delete(id: &str) -> MMCResult<String> {
        sample_client().delete(Endpoints::Asset, id)
    }

    fn random_id() -> String {
        Uuid::new_v4().hyphenated().to_string()
    }

    fn mock_single(endpoint: &str, id: &str, params: Option<&str>) -> Mock {
        mock(
            "GET",
            vec!["/", endpoint, "/", id, "/", params.unwrap_or("")]
                .join("")
                .as_str(),
        )
    }

    fn mock_create(parent: &str, p_id: &str, endpoint: &str) -> Mock {
        mock(
            "POST",
            vec!["/", parent, "/", p_id, "/", endpoint, "/"]
                .join("")
                .as_str(),
        )
    }

    fn mock_edit(endpoint: &str, id: &str) -> Mock {
        mock(
            "GET",
            vec!["/", endpoint, "/", id, "/edit/"].join("").as_str(),
        )
    }

    fn mock_update(endpoint: &str, id: &str) -> Mock {
        mock("PATCH", vec!["/", endpoint, "/", id, "/"].join("").as_str())
    }

    fn mock_asset_update(endpoint: &str, id: &str) -> Mock {
        mock(
            "PATCH",
            vec!["/", endpoint, "/", id, "/edit/"].join("").as_str(),
        )
    }

    fn mock_delete(endpoint: &str, id: &str) -> Mock {
        mock(
            "DELETE",
            vec!["/", endpoint, "/", id, "/edit/"].join("").as_str(),
        )
    }

    fn mock_list(endpoint: &str, param_string: &str) -> Mock {
        mock(
            "GET",
            vec!["/", endpoint, "/", param_string].join("").as_str(),
        )
    }

    #[test]
    fn single_200() {
        let id = random_id();
        let m = mock_single("shows", id.as_str(), None)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"value\"}")
            .create();

        let resp = show_get(id.as_str(), None);

        assert_eq!(resp.unwrap(), "{\"name\":\"value\"}");

        m.assert();
    }

    #[test]
    fn single_with_params_200() {
        let id = random_id();
        let param_string = "?param1=value1&param2=value2";

        let m = mock_single("shows", id.as_str(), Some(param_string))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"value\"}")
            .create();

        let resp = show_get(
            id.as_str(),
            Some(vec![("param1", "value1"), ("param2", "value2")]),
        );

        assert_eq!(resp.unwrap(), "{\"name\":\"value\"}");

        m.assert();
    }

    #[test]
    fn list_200() {
        let param_string = "?param1=value1&param2=value2";

        let m = mock_list("shows", param_string)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"value\"}")
            .create();

        let resp = show_list(vec![("param1", "value1"), ("param2", "value2")]);

        assert_eq!(resp.unwrap(), "{\"name\":\"value\"}");

        m.assert();
    }

    #[test]
    fn basic_auth_ok() {
        let id = random_id();
        let mut param_string = "?param=".to_string();
        param_string.push_str(id.as_str());

        let m = mock_list("shows", param_string.as_str())
            .match_header("Authorization", BASIC_AUTH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"value\"}")
            .create();

        let _ = show_list(vec![("param", id.as_str())]);

        m.assert();
    }

    #[test]
    fn get_400() {
        let id = random_id();
        let m = mock_single("shows", id.as_str(), None)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body("Failure message from the server")
            .create();

        let resp = show_get(id.as_str(), None);

        match resp.unwrap_err() {
            MMCError::BadRequest(msg) => {
                assert_eq!(msg, "Failure message from the server");
            }
            err => panic!("Expected BadRequest error but recieved {:?}", err),
        }

        m.assert();
    }

    #[test]
    fn get_401() {
        let id = random_id();
        let m = mock_single("shows", id.as_str(), None)
            .with_status(401)
            .create();

        let resp = show_get(id.as_str(), None);

        match resp.unwrap_err() {
            MMCError::NotAuthorized => (),
            err => panic!("Expected NotAuthorized error but recieved {:?}", err),
        }

        m.assert();
    }

    #[test]
    fn get_403() {
        let id = random_id();
        let m = mock_single("shows", id.as_str(), None)
            .with_status(403)
            .create();

        let resp = show_get(id.as_str(), None);

        match resp.unwrap_err() {
            MMCError::NotAuthorized => (),
            err => panic!("Expected NotAuthorized error but recieved {:?}", err),
        }

        m.assert();
    }

    #[test]
    fn get_404() {
        let id = random_id();
        let m = mock_single("shows", id.as_str(), None)
            .with_status(404)
            .create();

        let resp = show_get(id.as_str(), None);

        match resp.unwrap_err() {
            MMCError::ResourceNotFound => (),
            err => panic!("Expected ResourceNotFound error but recieved {:?}", err),
        }

        m.assert();
    }

    #[test]
    fn get_500() {
        let id = random_id();
        let m = mock_single("shows", id.as_str(), None)
            .with_status(500)
            .create();

        let resp = show_get(id.as_str(), None);

        match resp.unwrap_err() {
            MMCError::APIFailure(StatusCode::InternalServerError) => (),
            err => panic!("Expected APIFailure error but recieved {:?}", err),
        }

        m.assert();
    }

    #[test]
    fn shorthand_singles_200() {
        let id = random_id();

        let endpoints = vec![
            Endpoints::Asset,
            Endpoints::Collection,
            Endpoints::Episode,
            Endpoints::Franchise,
            Endpoints::Season,
            Endpoints::Special,
            Endpoints::Show,
        ];

        for endpoint in endpoints.into_iter() {
            let m = mock_single(endpoint.to_string().as_str(), id.as_str(), None)
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("{\"name\":\"value\"}")
                .create();

            let resp = sample_client().get(endpoint.clone(), id.as_str(), None);

            assert_eq!(resp.unwrap(), String::from("{\"name\":\"value\"}"));

            m.assert();
        }
    }

    #[test]
    fn shorthand_list_200() {
        let id = random_id();

        let param_string = vec!["?param1=", id.as_str(), "&param2=value2"].join("");
        let params = vec![("param1", id.as_str()), ("param2", "value2")];

        let endpoints = vec![
            Endpoints::Changelog,
            Endpoints::Collection,
            Endpoints::Franchise,
            Endpoints::Show,
        ];

        for endpoint in endpoints.into_iter() {
            let m = mock_list(endpoint.to_string().as_str(), param_string.as_str())
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("{\"name\":\"value\"}")
                .create();

            let resp = sample_client().list(endpoint.clone(), params.clone());
            assert_eq!(resp.unwrap(), String::from("{\"name\":\"value\"}"));

            m.assert();
        }
    }

    #[test]
    fn create_204() {
        let p_id = random_id();

        let m = mock_create("shows", p_id.as_str(), "assets")
            .with_status(204)
            .with_header("content-type", "application/json")
            .with_body("")
            .match_body("{}")
            .create();

        let _ = show_create(p_id.as_str(), &EmptyReq {});

        m.assert();
    }

    #[test]
    fn create_400() {
        let p_id = random_id();
        let body = "{\"name\":\"value\"}";
        let server_error = "Payload missing parameter";

        let m = mock_create("shows", p_id.as_str(), "assets")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(server_error)
            .create();

        let resp = show_create(p_id.as_str(), &body);

        match resp.unwrap_err() {
            MMCError::BadRequest(err) => {
                assert_eq!(err, String::from(server_error));
            }
            err => panic!("Expected BadRequest error but recieved {:?}", err),
        }

        m.assert();
    }

    #[test]
    fn edit_200() {
        let id = random_id();
        let body = "{\"name\":\"value\"}";

        let m = mock_edit("assets", id.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        let resp = show_edit(id.as_str());
        assert_eq!(resp.unwrap(), body);

        m.assert();
    }

    #[test]
    fn update_200() {
        let id = random_id();

        let m = mock_asset_update("assets", id.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .match_body("{}")
            .create();

        let _ = show_update(id.as_str(), &EmptyReq {});

        m.assert();
    }

    #[test]
    fn delete_200() {
        let id = random_id();

        let m = mock_delete("assets", id.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .match_body("")
            .create();

        let _ = show_delete(id.as_str());

        m.assert();
    }

    #[test]
    fn move_special_to_season() {
        let special_id = random_id();
        let season_id = random_id();
        let body_str = [
            "{\"data\":{\"type\":\"special\",\"id\":\"",
            special_id.as_str(),
            "\",\"attributes\":{\"season\":\"",
            season_id.as_str(),
            "\"}}}",
        ]
        .join("");

        let m = mock_update("specials", special_id.as_str())
            .with_status(204)
            .with_header("content-type", "application/json")
            .with_body("")
            .match_body(body_str.as_str())
            .create();

        let _ = sample_client().change_parent(
            Endpoints::Season,
            season_id.as_str(),
            Endpoints::Special,
            special_id.as_str(),
        );

        m.assert();
    }
}
