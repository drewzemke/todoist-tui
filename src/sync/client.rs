use super::{Request, Response};
use anyhow::Result;

pub struct Client {
    client: reqwest::Client,
    sync_url: String,
    api_token: String,
}

impl Client {
    #[must_use]
    pub fn new(sync_url: String, api_token: String) -> Self {
        Client {
            sync_url,
            api_token,
            client: reqwest::Client::new(),
        }
    }

    /// # Errors
    ///
    /// Returns an error if the request fails (for network reasons) or if the returned data
    /// cannot be parsed as expected.
    // TODO: Better error handling, specifically to handle the various types of errors that can
    // come back from the Sync API
    pub async fn make_request(&self, request: &Request) -> Result<Response> {
        let client_response = self
            .client
            .post(format!("{}/sync", self.sync_url))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&request)
            .send()
            .await?;
        let parsed_response = client_response.json().await?;
        Ok(parsed_response)
    }
}
