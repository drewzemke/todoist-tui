#[cfg(test)]
pub use wiremock_wrapper::ApiMockBuilder;

#[cfg(test)]
mod wiremock_wrapper {
    use serde::{Deserialize, Serialize};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    pub struct ApiMockBuilder {
        mock_server: MockServer,
    }

    impl ApiMockBuilder {
        pub async fn new() -> Self {
            ApiMockBuilder {
                mock_server: MockServer::start().await,
            }
        }

        // HACK: Not sure if the typing on `F` here is all necessary, or if there's a way around the `Clone` constraint
        pub async fn mock_response<F, T, R>(self, condition: F, response: R) -> Self
        where
            F: Fn(T) -> bool + Send + Sync + Clone + 'static,
            T: for<'de> Deserialize<'de>,
            R: Serialize,
        {
            let matcher =
                move |request: &Request| request.body_json::<T>().is_ok_and(condition.clone());
            let response = ResponseTemplate::new(200).set_body_json(response);
            Mock::given(matcher)
                .respond_with(response)
                .mount(&self.mock_server)
                .await;
            self
        }

        pub fn uri(&self) -> String {
            self.mock_server.uri()
        }
    }
}
