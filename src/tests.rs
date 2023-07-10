mod utils;

#[cfg(test)]
pub mod e2e {
    use crate::{
        sync::{
            AddItemRequest, GetUserRequest, Item, Project, ProjectDataRequest, ProjectDataResponse,
            Response, User,
        },
        tests::utils::{ApiMockBuilder, FsMockBuilder},
    };
    use assert_cmd::Command;
    use std::collections::HashMap;

    #[tokio::test]
    async fn add_to_inbox_when_user_data_exists() -> Result<(), Box<dyn std::error::Error>> {
        // create mock `client_auth.toml` and `data/user.json`
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_key = \"MOCK_API_KEY\"")?
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
                "sync",
                |request: AddItemRequest| {
                    request.commands[0].args.project_id == "MOCK_INBOX_PROJECT_ID"
                },
                Response {
                    full_sync: true,
                    sync_status: None,
                    sync_token: String::from("MOCK_SYNC_TOKEN"),
                    temp_id_mapping: HashMap::new(),
                    user: None,
                },
            )
            .await;
        let server_url = mock_server.uri();

        // run the thing
        let mut cmd =
            Command::cargo_bin("todoist").expect("could not run program using 'assert_cmd'");
        cmd.arg("--local-dir").arg(mock_data_dir);
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
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_key = \"MOCK_API_KEY\"")?;
        let mock_data_dir = mock_fs.path();

        // set up mock server
        let mock_server = ApiMockBuilder::new()
            .await
            .mock_response(
                "sync",
                |request: GetUserRequest| {
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
                },
            )
            .await
            .mock_response(
                "sync",
                |request: AddItemRequest| {
                    request
                        .commands
                        .get(0)
                        .is_some_and(|command| command.args.project_id == "MOCK_INBOX_PROJECT_ID")
                },
                Response {
                    full_sync: true,
                    sync_status: None,
                    sync_token: String::from("MOCK_SYNC_TOKEN"),
                    temp_id_mapping: HashMap::new(),
                    user: None,
                },
            )
            .await;
        let server_url = mock_server.uri();

        // run the thing
        let mut cmd = Command::cargo_bin("todoist")
            .map_err(|err| format!("Could not run app using 'assert_cmd': {err:?}"))?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("--add").arg("new todo!");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Fetching user data"))
            .stdout(predicates::str::contains("Storing user data"))
            .stdout(predicates::str::contains("Todo 'new todo!' added"));

        // check that a file was created
        assert!(mock_data_dir.join("data").join("user.json").exists());

        Ok(())
    }

    #[tokio::test]
    async fn get_inbox_items() -> Result<(), Box<dyn std::error::Error>> {
        // create mock `client_auth.toml` and `data/user.json`
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_key = \"MOCK_API_KEY\"")?
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
                            content: "Todo One!".to_string(),
                        },
                        Item {
                            id: "MOCK_ITEM_ID_2".to_string(),
                            content: "Todo Two!".to_string(),
                        },
                        Item {
                            id: "MOCK_ITEM_ID_3".to_string(),
                            content: "Todo Three!".to_string(),
                        },
                    ],
                },
            )
            .await;
        let server_url = mock_server.uri();

        // run the thing
        let mut cmd =
            Command::cargo_bin("todoist").expect("could not run program using 'assert_cmd'");
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("--list");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("- Todo One!"))
            .stdout(predicates::str::contains("- Todo Two!"))
            .stdout(predicates::str::contains("- Todo Three!"));

        Ok(())
    }
}
