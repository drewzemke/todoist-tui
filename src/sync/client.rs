use super::{Request, Response};
use anyhow::Result;

const SYNC_URL: &str = "https://api.todoist.com/sync/v9";

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    sync_url: String,
    api_token: String,
}

impl Client {
    #[must_use]
    pub fn new(api_token: &str, sync_url_override: Option<&str>) -> Self {
        Client {
            sync_url: sync_url_override
                .map_or(SYNC_URL.to_string(), std::string::ToString::to_string),
            api_token: api_token.to_string(),
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
