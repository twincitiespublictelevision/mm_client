#[macro_use]
#[cfg(test)]
extern crate assert_matches;
#[cfg(test)]
extern crate mockito;
#[cfg(test)]
extern crate uuid;

mod client;
mod error;
mod request;
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

    fn sample_client() -> Client<'static> {
        Client::qa("", "")
    }

    fn episode_get(id: &str) -> CDCResult<String> {
        sample_client().get(Endpoints::Episode, id)
    }

    fn episode_list(params: Params) -> CDCResult<String> {
        sample_client().list(Endpoints::Episode, params)
    }

    fn random_id() -> String {
        Uuid::new_v4().hyphenated().to_string()
    }

    fn mock_episode_endpoint(id: &str) -> Mock {
        mock("GET", vec!["/episodes/", id, "/"].join("").as_str())
    }

    fn mock_episodes_endpoint(param_string: &str) -> Mock {
        mock("GET", vec!["/episodes/", param_string].join("").as_str())
    }

    #[test]
    fn handles_single_200() {
        let id = random_id();
        let mock = mock_episode_endpoint(id.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"value\"}")
            .create_for(|| {
                let test_response = String::from("{\"name\":\"value\"}");
                assert_matches!(episode_get(id.as_str()), Ok(test_response))
            })
            .remove();
    }

    #[test]
    fn handles_list_200() {
        let param_string = "?param1=value1&param2=value2";

        let mock = mock_episodes_endpoint(param_string)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"name\":\"value\"}")
            .create_for(|| {
                let params = vec![("param1", "value1"), ("param2", "value2")];
                let test_response = String::from("{\"name\":\"value\"}");
                assert_matches!(episode_list(params), Ok(test_response))
            })
            .remove();
    }

    #[test]
    fn handles_400() {
        let id = random_id();
        let mock = mock_episode_endpoint(id.as_str())
            .with_status(400)
            .with_header("content-type", "text/plain")
            .with_body("Failure message from the server")
            .create_for(|| {
                let bad_rq_error = CDCError::BadRequest(String::from("Failure message from the \
                                                                      server"));

                assert_matches!(episode_get(id.as_str()), Err(bad_rq_error))
            })
            .remove();
    }

    #[test]
    fn handles_401() {
        let id = random_id();
        let mock = mock_episode_endpoint(id.as_str())
            .with_status(401)
            .create_for(|| assert_matches!(episode_get(id.as_str()), Err(CDCError::NotAuthorized)))
            .remove();
    }

    #[test]
    fn handles_403() {
        let id = random_id();
        let mock = mock_episode_endpoint(id.as_str())
            .with_status(403)
            .create_for(|| assert_matches!(episode_get(id.as_str()), Err(CDCError::NotAuthorized)))
            .remove();
    }

    #[test]
    fn handles_404() {
        let id = random_id();
        let mock = mock_episode_endpoint(id.as_str())
            .with_status(404)
            .create_for(|| {
                assert_matches!(episode_get(id.as_str()), Err(CDCError::ResourceNotFound))
            })
            .remove();
    }

    #[test]
    fn handles_500() {
        let id = random_id();
        let mock = mock_episode_endpoint(id.as_str())
            .with_status(500)
            .create_for(|| assert_matches!(episode_get(id.as_str()), Err(CDCError::APIFailure)))
            .remove();
    }
}
