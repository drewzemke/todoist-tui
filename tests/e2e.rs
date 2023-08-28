#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
mod test_utils;

#[cfg(test)]
pub mod e2e {
    use std::{collections::HashMap, fs};
    use todoist::sync::{
        self, AddItemCommandArgs, CommandArgs, Item, Project, ProjectDataRequest,
        ProjectDataResponse, Request, Response, User,
    };

    use crate::test_utils::{ApiMockBuilder, FsMockBuilder};

    #[tokio::test]
    // Ignoring for now, since we're changing the behavior to use incremental sync
    #[ignore]
    async fn add_to_inbox_when_user_data_exists() -> Result<(), Box<dyn std::error::Error>> {
        // create mock `client_auth.toml` and `data/user.json`
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_token = \"MOCK_API_TOKEN\"")?
            .mock_file_contents(
                "data/user.json",
                r#"{
                    "full_name": "Drew",
                    "inbox_project_id": "MOCK_INBOX_PROJECT_ID"     
                }"#,
            )?;
        let mock_data_dir = mock_fs.path();

        // set up mock server
        let mock_server =
            ApiMockBuilder::new()
                .await
                .mock_response(
                    "sync",
                    |request: Request| match &request.commands[0] {
                        sync::Command {
                            args:
                                CommandArgs::AddItemCommandArgs(AddItemCommandArgs {
                                    project_id, ..
                                }),
                            ..
                        } => project_id == "MOCK_INBOX_PROJECT_ID",
                        _ => false,
                    },
                    Response {
                        full_sync: true,
                        sync_status: None,
                        sync_token: String::from("MOCK_SYNC_TOKEN"),
                        temp_id_mapping: HashMap::new(),
                        user: None,
                        items: vec![],
                    },
                )
                .await;
        let server_url = mock_server.uri();

        // run the thing
        let mut cmd = assert_cmd::Command::cargo_bin("todoist")
            .expect("could not run program using 'assert_cmd'");
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("add").arg("new todo!");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Todo 'new todo!' added"));

        Ok(())
    }

    #[tokio::test]
    // Ignoring for now, since we're changing the behavior to use incremental sync
    #[ignore]
    async fn add_to_inbox_when_user_data_missing() -> Result<(), Box<dyn std::error::Error>> {
        // create mock `client_auth.toml`
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_token = \"MOCK_API_TOKEN\"")?;
        let mock_data_dir = mock_fs.path();

        // set up mock server
        let mock_server =
            ApiMockBuilder::new()
                .await
                .mock_response(
                    "sync",
                    |request: Request| {
                        request
                            .resource_types
                            .get(0)
                            .is_some_and(|resource| resource == "user")
                    },
                    Response {
                        full_sync: true,
                        sync_status: None,
                        sync_token: String::from("MOCK_SYNC_TOKEN"),
                        temp_id_mapping: HashMap::new(),
                        user: Some(User {
                            full_name: "Drew".to_string(),
                            inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                        }),
                        items: vec![],
                    },
                )
                .await
                .mock_response(
                    "sync",
                    |request: Request| match &request.commands[0] {
                        sync::Command {
                            args:
                                CommandArgs::AddItemCommandArgs(AddItemCommandArgs {
                                    project_id, ..
                                }),
                            ..
                        } => project_id == "MOCK_INBOX_PROJECT_ID",
                        _ => false,
                    },
                    Response {
                        full_sync: true,
                        sync_status: None,
                        sync_token: String::from("MOCK_SYNC_TOKEN"),
                        temp_id_mapping: HashMap::new(),
                        user: None,
                        items: vec![],
                    },
                )
                .await;
        let server_url = mock_server.uri();

        // run the thing
        let mut cmd = assert_cmd::Command::cargo_bin("todoist")
            .map_err(|err| format!("Could not run app using 'assert_cmd': {err:?}"))?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("add").arg("new todo!");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Fetching user data"))
            .stdout(predicates::str::contains("Stored user data"))
            .stdout(predicates::str::contains("Todo 'new todo!' added"));

        // check that a file was created
        assert!(mock_data_dir.join("data").join("user.json").exists());

        Ok(())
    }

    #[test]
    fn print_error_when_api_token_missing() -> Result<(), Box<dyn std::error::Error>> {
        // no need to mock the server or file dir, but still going to use fakes for both
        // to prevent accidental calls to the real api or filesystem
        let mock_fs = FsMockBuilder::new()?;
        let mock_data_dir = mock_fs.path();
        let server_url = "fake/server/url";

        // run the thing
        let mut cmd = assert_cmd::Command::cargo_bin("todoist")
            .map_err(|err| format!("Could not run app using 'assert_cmd': {err:?}"))?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("sync");

        // check output
        cmd.assert()
            .stderr(predicates::str::contains("Could not find an API token"))
            .stderr(predicates::str::contains("re-run"));

        Ok(())
    }

    #[test]
    fn store_api_token() -> Result<(), Box<dyn std::error::Error>> {
        // empty directory
        let mock_fs = FsMockBuilder::new()?;
        let mock_data_dir = mock_fs.path();

        // no need to mock the server, but still going to use a fake url to prevent
        // accidental calls to the real api
        let server_url = "fake/server/url";

        // run the thing
        let mut cmd = assert_cmd::Command::cargo_bin("todoist")
            .map_err(|err| format!("Could not run app using 'assert_cmd': {err:?}"))?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("set-token").arg("MOCK_API_TOKEN");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Stored API token"));

        // check that a file was created with the correct content
        assert!(mock_data_dir.join("client_auth.toml").exists());
        let file_contents = fs::read_to_string(mock_data_dir.join("client_auth.toml"))?;
        let file_contents: toml::Value = toml::from_str(&file_contents)?;
        assert_eq!(
            file_contents["api_token"],
            toml::Value::from("MOCK_API_TOKEN")
        );
        Ok(())
    }

    #[tokio::test]
    // Ignoring for now, since we're changing the behavior to use incremental sync
    #[ignore]
    async fn get_inbox_items() -> Result<(), Box<dyn std::error::Error>> {
        // create mock `client_auth.toml` and `data/user.json`
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_token = \"MOCK_API_TOKEN\"")?
            .mock_file_contents(
                "data/user.json",
                r#"{
                    "full_name": "Drew",
                    "inbox_project_id": "MOCK_INBOX_PROJECT_ID"     
                }"#,
            )?;
        let mock_data_dir = mock_fs.path();

        // set up mock server
        let mock_server = ApiMockBuilder::new()
            .await
            .mock_response(
                "projects/get_data",
                |request: ProjectDataRequest| request.project_id == "MOCK_INBOX_PROJECT_ID",
                ProjectDataResponse {
                    project: Project {
                        id: "MOCK_INBOX_PROJECT_ID".to_string(),
                        name: "Inbox".to_string(),
                    },
                    items: vec![
                        Item {
                            id: "MOCK_ITEM_ID_1".to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo One!".to_string(),
                        },
                        Item {
                            id: "MOCK_ITEM_ID_2".to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo Two!".to_string(),
                        },
                        Item {
                            id: "MOCK_ITEM_ID_3".to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo Three!".to_string(),
                        },
                    ],
                },
            )
            .await;
        let server_url = mock_server.uri();

        // run the thing
        let mut cmd = assert_cmd::Command::cargo_bin("todoist")
            .expect("could not run program using 'assert_cmd'");
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("list");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("- Todo One!"))
            .stdout(predicates::str::contains("- Todo Two!"))
            .stdout(predicates::str::contains("- Todo Three!"));

        Ok(())
    }
}
