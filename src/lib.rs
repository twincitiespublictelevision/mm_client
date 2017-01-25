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
pub use error::CDCResult;
pub use error::CDCError;

#[cfg(test)]
mod tests {
    use client::Client;
    use client::Params;
    use client::Endpoints;
    use error::CDCResult;
    use error::CDCError;
    use mockito::mock;
    use mockito::Mock;
    use uuid::Uuid;
    use reqwest::StatusCode;

    const KEY: &'static str = "hello";
    const SECRET: &'static str = "world";
    const BASIC_AUTH: &'static str = "Basic aGVsbG86d29ybGQ=";

    fn sample_client() -> Client<'static> {
        Client::qa(KEY, SECRET).unwrap()
    }

    fn show_get(id: &str) -> CDCResult<String> {
        sample_client().get(Endpoints::Show, id)
    }

    fn show_list(params: Params) -> CDCResult<String> {
        sample_client().list(Endpoints::Show, params)
    }

    fn random_id() -> String {
        Uuid::new_v4().hyphenated().to_string()
    }

    fn mock_show_endpoint(id: &str) -> Mock {
        mock("GET", vec!["/shows/", id, "/"].join("").as_str())
    }

    fn mock_shows_endpoint(param_string: &str) -> Mock {
        mock("GET", vec!["/shows/", param_string].join("").as_str())
    }

    #[test]
    fn handles_single_200() {
        let id = random_id();
        let mock = mock_show_endpoint(id.as_str())
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

        let mock = mock_shows_endpoint(param_string)
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

        let mock = mock_shows_endpoint(param_string.as_str())
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
        let mock = mock_show_endpoint(id.as_str())
            .with_status(400)
            .with_header("content-type", "text/plain")
            .with_body("Failure message from the server")
            .create_for(|| {
                let bad_rq_error = CDCError::BadRequest(String::from("Failure message from the \
                                                                      server"));

                assert_matches!(show_get(id.as_str()), Err(bad_rq_error))
            })
            .remove();
    }

    #[test]
    fn handles_401() {
        let id = random_id();
        let mock = mock_show_endpoint(id.as_str())
            .with_status(401)
            .create_for(|| assert_matches!(show_get(id.as_str()), Err(CDCError::NotAuthorized)))
            .remove();
    }

    #[test]
    fn handles_403() {
        let id = random_id();
        let mock = mock_show_endpoint(id.as_str())
            .with_status(403)
            .create_for(|| assert_matches!(show_get(id.as_str()), Err(CDCError::NotAuthorized)))
            .remove();
    }

    #[test]
    fn handles_404() {
        let id = random_id();
        let mock = mock_show_endpoint(id.as_str())
            .with_status(404)
            .create_for(|| assert_matches!(show_get(id.as_str()), Err(CDCError::ResourceNotFound)))
            .remove();
    }

    #[test]
    fn handles_500() {
        let id = random_id();
        let mock = mock_show_endpoint(id.as_str())
            .with_status(500)
            .create_for(|| {
                assert_matches!(show_get(id.as_str()),
                                Err(CDCError::APIFailure(StatusCode::InternalServerError)))
            })
            .remove();
    }
}
