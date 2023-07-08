#[cfg(test)]
pub mod e2e {
    use assert_cmd::Command;
    use assert_fs::prelude::{FileTouch, FileWriteStr, PathChild};
    use std::collections::HashMap;
    use std::fs;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use crate::sync::{AddItemSyncRequest, GetUserSyncRequest, SyncResponse, User};

    #[tokio::test]
    async fn add_to_inbox_when_user_data_exists() -> Result<(), Box<dyn std::error::Error>> {
        // create mock `client_auth.toml` and `data/user.json`
        let mock_local_dir = assert_fs::TempDir::new().expect("could not create mock directory");
        let data_dir = mock_local_dir.path();

        let mock_client_config = mock_local_dir.child("client_auth.toml");
        mock_client_config.touch().unwrap();
        mock_client_config
            .write_str("api_key = \"MOCK_API_KEY\"")
            .unwrap();

        let mock_user_storage = mock_local_dir.child("data").child("user.json");
        mock_user_storage.touch().unwrap();
        mock_user_storage
            .write_str(
                r#"{
                    "full_name": "Drew",
                    "inbox_project_id": "MOCK_INBOX_PROJECT_ID"     
                }"#,
            )
            .unwrap();

        // set up mock server
        let mock_server = MockServer::start().await;
        let server_url = mock_server.uri();

        let matcher = |request: &Request| {
            request
                .body_json::<AddItemSyncRequest>()
                .is_ok_and(|request| request.commands[0].args.project_id == "MOCK_INBOX_PROJECT_ID")
        };

        let template = {
            let response = SyncResponse {
                full_sync: true,
                sync_status: None,
                sync_token: String::from("MOCK_SYNC_TOKEN"),
                temp_id_mapping: HashMap::new(),
                user: None,
            };
            ResponseTemplate::new(200).set_body_json(response)
        };

        Mock::given(matcher)
            .respond_with(template)
            .mount(&mock_server)
            .await;

        // run the thing
        let mut cmd = Command::cargo_bin("todoist").unwrap();
        cmd.arg("--local-dir").arg(data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("--add").arg("new todo!");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Todo 'new todo!' added"));

        Ok(())
    }

    #[tokio::test]
    async fn add_to_inbox_when_user_data_missing() -> Result<(), Box<dyn std::error::Error>> {
        // create mock `client_auth.toml`
        let mock_local_dir = assert_fs::TempDir::new().expect("could not create mock directory");
        let data_dir = mock_local_dir.path();

        let mock_client_config = mock_local_dir.child("client_auth.toml");
        mock_client_config.touch().unwrap();
        mock_client_config
            .write_str("api_key = \"MOCK_API_KEY\"")
            .unwrap();

        // set up mock server
        let mock_server = MockServer::start().await;
        let server_url = mock_server.uri();

        let get_user_matcher = |request: &Request| {
            request
                .body_json::<GetUserSyncRequest>()
                .is_ok_and(|request| {
                    request
                        .resource_types
                        .get(0)
                        .is_some_and(|resource| resource == "user")
                })
        };

        let get_user_response = {
            let response = SyncResponse {
                full_sync: true,
                sync_status: None,
                sync_token: String::from("MOCK_SYNC_TOKEN"),
                temp_id_mapping: HashMap::new(),
                user: Some(User {
                    full_name: "Drew".to_string(),
                    inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                }),
            };
            ResponseTemplate::new(200).set_body_json(response)
        };

        Mock::given(get_user_matcher)
            .respond_with(get_user_response)
            .mount(&mock_server)
            .await;

        let add_todo_matcher = |request: &Request| {
            request
                .body_json::<AddItemSyncRequest>()
                .is_ok_and(|request| {
                    request
                        .commands
                        .get(0)
                        .is_some_and(|command| command.args.project_id == "MOCK_INBOX_PROJECT_ID")
                })
        };

        let add_todo_response = {
            let response = SyncResponse {
                full_sync: true,
                sync_status: None,
                sync_token: String::from("MOCK_SYNC_TOKEN"),
                temp_id_mapping: HashMap::new(),
                user: None,
            };
            ResponseTemplate::new(200).set_body_json(response)
        };

        Mock::given(add_todo_matcher)
            .respond_with(add_todo_response)
            .mount(&mock_server)
            .await;

        // run the thing
        let mut cmd = Command::cargo_bin("todoist").unwrap();
        cmd.arg("--local-dir").arg(data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("--add").arg("new todo!");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Fetching user data"))
            .stdout(predicates::str::contains("Storing user data"))
            .stdout(predicates::str::contains("Todo 'new todo!' added"));

        // check that a file was created
        assert!(data_dir.join("data").join("user.json").exists());

        Ok(())
    }

    #[tokio::test]
    #[ignore]
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
}
