#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
mod test_utils;

#[cfg(test)]
pub mod sync {
    use assert_cmd::Command;
    use std::{collections::HashMap, fs};
    use todoist::sync::{self, AddItemCommand, AddItemRequestArgs, Item, Request, Response, User};
    use uuid::Uuid;

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

    #[test]
    fn add_todo_to_local() -> Result<(), Box<dyn std::error::Error>> {
        // create mock and `data/sync.json`
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
        cmd.arg("add").arg("new todo!");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Todo 'new todo!' added"));

        // check that the commands file was created with the correct content
        let commands_file = mock_data_dir.join("data").join("commands.json");
        assert!(commands_file.exists());

        let file_contents = fs::read_to_string(commands_file)?;
        let commands: Vec<sync::Command> = serde_json::from_str(&file_contents)?;
        assert_eq!(commands.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn incremental_sync_send_new_todo() -> Result<(), Box<dyn std::error::Error>> {
        let new_item_temp_id = Uuid::new_v4();

        // create mock `data/sync.json` and `data/commands.json`
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_token = \"MOCK_API_TOKEN\"")?
            .mock_file_contents(
                "data/user.json",
                r#"{
                    "full_name": "Drew",
                    "inbox_project_id": "MOCK_INBOX_PROJECT_ID"     
                }"#,
            )?
            .mock_file_contents(
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
                            id: new_item_temp_id.to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo Two!".to_string(),
                        },
                    ],
                })?,
            )?
            .mock_file_contents(
                "data/commands.json",
                serde_json::to_string_pretty(&[&sync::Command::AddItem(AddItemCommand {
                    request_type: "item_add".to_owned(),
                    temp_id: new_item_temp_id,
                    uuid: Uuid::new_v4(),
                    args: AddItemRequestArgs {
                        project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                        content: "Todo Two!".to_string(),
                    },
                })])?,
            )?;
        let mock_data_dir = mock_fs.path();

        // set up mock server
        let mock_server = ApiMockBuilder::new()
            .await
            .mock_response(
                "sync",
                |request: Request| {
                    request.sync_token == "MOCK_SYNC_TOKEN"
                        && request.resource_types.get(0).is_some_and(|s| s == "all")
                },
                Response {
                    full_sync: false,
                    sync_token: String::from("NEW_MOCK_SYNC_TOKEN"),
                    sync_status: Some(HashMap::from([("UUID".to_string(), "ok".to_string())])),
                    temp_id_mapping: HashMap::from([(
                        new_item_temp_id,
                        "MOCK_ITEM_ID_2_NEW".to_string(),
                    )]),
                    user: None,
                    items: vec![],
                },
            )
            .await;
        let server_url = mock_server.uri();

        let mut cmd = assert_cmd::Command::cargo_bin("todoist")?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("sync");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Syncing latest changes"))
            .stdout(predicates::str::contains("Stored sync data"));

        // check that the sync data file was updated with the correct content
        let sync_file = mock_data_dir.join("data").join("sync.json");
        let file_contents = fs::read_to_string(sync_file)?;
        let sync_data: Response = serde_json::from_str(&file_contents)?;

        assert_eq!(sync_data.sync_token, "NEW_MOCK_SYNC_TOKEN");
        assert_eq!(sync_data.items.len(), 2);
        assert_eq!(sync_data.items[0].id, "MOCK_ITEM_ID_1");
        assert_eq!(sync_data.items[1].id, "MOCK_ITEM_ID_2_NEW");

        // check that the commands file is now empty
        let commands_file = mock_data_dir.join("data").join("commands.json");
        assert!(commands_file.exists());

        let file_contents = fs::read_to_string(commands_file)?;
        let commands: Vec<sync::Command> = serde_json::from_str(&file_contents)?;
        assert_eq!(commands.len(), 0);

        Ok(())
    }

    // TODO: next up! incremental sync, receiving (and processing) a new todo that was added remotely
}
