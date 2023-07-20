#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
mod test_utils;

#[cfg(test)]
pub mod sync {
    use assert_cmd::Command;
    use std::collections::HashMap;
    use todoist::sync::{Item, Request, Response, User};

    use crate::test_utils::{ApiMockBuilder, FsMockBuilder};

    #[tokio::test]
    async fn full_sync_and_store_data() -> Result<(), Box<dyn std::error::Error>> {
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
                "sync",
                |request: Request| {
                    request.sync_token == "*"
                        && request.resource_types.get(0).is_some_and(|s| s == "all")
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
                    ],
                },
            )
            .await;
        let server_url = mock_server.uri();

        // run the thing
        let mut cmd = Command::cargo_bin("todoist")?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("sync");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Performing a full sync"))
            .stdout(predicates::str::contains("Stored sync data"));

        // check that a file was created
        assert!(mock_data_dir.join("data").join("sync.json").exists());

        Ok(())
    }

    #[test]
    fn get_inbox_items_from_local() -> Result<(), Box<dyn std::error::Error>> {
        // create mock `data/sync.json`
        let mock_fs = FsMockBuilder::new()?.mock_file_contents(
            "data/sync.json",
            // HACK: wrong data type, need a common storage type
            serde_json::to_string_pretty(&Response {
                full_sync: true,
                sync_status: None,
                sync_token: String::from("MOCK_SYNC_TOKEN"),
                temp_id_mapping: HashMap::new(),
                user: Some(User {
                    full_name: "Drew".to_string(),
                    inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                }),
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
                ],
            })?,
        )?;
        let mock_data_dir = mock_fs.path();

        // no need to mock the server, but still going to use a fake url to prevent
        // accidental calls to the real api
        let server_url = "fake/server/url";

        let mut cmd = assert_cmd::Command::cargo_bin("todoist")?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("list");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("- Todo One!"))
            .stdout(predicates::str::contains("- Todo Two!"));

        Ok(())
    }
}
