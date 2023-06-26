#[cfg(test)]
pub mod e2e {
    use assert_cmd::Command;
    use assert_fs::prelude::{FileTouch, FileWriteStr, PathChild};
    use serde::{Deserialize, Serialize};
    use std::fs;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SyncResponse {
        pub user: User,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct User {
        pub full_name: String,
        pub inbox_project_id: String,
    }

    #[tokio::test]
    async fn mock_user_request() -> Result<(), Box<dyn std::error::Error>> {
        // load fixture response from file
        let user_response: SyncResponse = {
            let path = "fixtures/user_response.json";
            let file_contents = fs::read_to_string(path).expect("could not read from fixture");
            serde_json::from_str(&file_contents).expect("could not parse fixture file into json")
        };

        // set up mock server
        let mock_server = MockServer::start().await;
        let server_url = mock_server.uri();

        let template = ResponseTemplate::new(200).set_body_json(user_response);
        Mock::given(method("POST"))
            .respond_with(template)
            .mount(&mock_server)
            .await;

        // run the thing
        let mut cmd = Command::cargo_bin("todoist").unwrap();
        cmd.arg("--sync-url").arg(server_url);

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("FIXTURE_INBOX_ID"));

        Ok(())
    }

    #[test]
    fn mock_api_key() -> Result<(), Box<dyn std::error::Error>> {
        // create a fixture file to load
        let mock_local_dir = assert_fs::TempDir::new().expect("could not create mock directory");
        let mock_client_config = mock_local_dir.child("client_auth.toml");
        mock_client_config.touch().unwrap();
        mock_client_config
            .write_str("api_key = \"MOCK_API_KEY\"")
            .unwrap();
        let data_dir = mock_local_dir.path();

        // run the thing
        let mut cmd = Command::cargo_bin("todoist").unwrap();
        cmd.arg("--local-dir").arg(data_dir);

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("MOCK_API_KEY"));

        Ok(())
    }
}
