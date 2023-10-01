#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
pub mod test_utils;

#[cfg(test)]
pub mod sync_tests {
    use anyhow::Result;
    use assert_cmd::Command;
    use std::{collections::HashMap, fs};
    use tod::{
        model::{
            command::{self, AddItemArgs, Args},
            item::Item,
            user::User,
            Model,
        },
        sync::{Request, Response, Status},
    };
    use uuid::Uuid;

    use crate::test_utils::{ApiMockBuilder, FsMockBuilder};

    #[tokio::test]
    async fn full_sync_and_store_data() -> Result<()> {
        // create mock `client_auth.toml`
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_token = \"MOCK_API_TOKEN\"")?;
        let mock_data_dir = mock_fs.path();

        // mock data
        let mock_item_1 = Item::new("Todo One!", "MOCK_INBOX_PROJECT_ID");
        let mock_item_2 = Item::new("Todo Two!", "MOCK_INBOX_PROJECT_ID");

        // set up mock server
        let mock_server = ApiMockBuilder::new()
            .await
            .mock_response(
                "sync",
                |request: Request| request.sync_token == "*",
                Response {
                    full_sync: true,
                    items: vec![mock_item_1, mock_item_2],
                    sync_status: None,
                    sync_token: String::from("MOCK_SYNC_TOKEN"),
                    temp_id_mapping: HashMap::new(),
                    user: Some(User {
                        full_name: "Drew".to_string(),
                        inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                    }),
                },
            )
            .await;
        let server_url = mock_server.uri();

        // run the thing
        let mut cmd = Command::cargo_bin("tod")?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("sync");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Syncing"))
            .code(0);

        // check that a file was created
        assert!(mock_data_dir.join("sync.json").exists());

        Ok(())
    }

    #[tokio::test]
    async fn full_sync_send_new_todo() -> Result<()> {
        // mock data
        let command_uuid = Uuid::new_v4();
        let mock_item_1 = Item::new("Todo One!", "MOCK_INBOX_PROJECT_ID");
        let mock_item_2 = Item::new("Todo Two!", "MOCK_INBOX_PROJECT_ID");
        let mock_item_3 = Item::new("Todo Three!", "MOCK_INBOX_PROJECT_ID");
        let mock_item_2_updated = Item::new("Todo Two!", "MOCK_INBOX_PROJECT_ID");

        // create mock `sync.json`
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_token = \"MOCK_API_TOKEN\"")?
            .mock_file_contents(
                "sync.json",
                serde_json::to_string_pretty(&Model {
                    sync_token: String::from("MOCK_SYNC_TOKEN"),
                    user: User {
                        full_name: "Drew".to_string(),
                        inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                    },
                    items: vec![mock_item_1.clone(), mock_item_2.clone()],
                    commands: vec![command::Command {
                        request_type: "item_add".to_owned(),
                        temp_id: Some(mock_item_2.id.clone()),
                        uuid: command_uuid,
                        args: Args::AddItemCommandArgs(AddItemArgs {
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo Two!".to_string(),
                        }),
                    }],
                })?,
            )?;
        let mock_data_dir = mock_fs.path();

        // set up mock server
        let mock_server = ApiMockBuilder::new()
            .await
            .mock_response(
                "sync",
                |request: Request| request.sync_token == "*",
                Response {
                    full_sync: true,
                    items: vec![
                        mock_item_1.clone(),
                        mock_item_2_updated.clone(),
                        mock_item_3.clone(),
                    ],
                    sync_status: Some(HashMap::from([(command_uuid, Status::Ok)])),
                    sync_token: String::from("NEW_MOCK_SYNC_TOKEN"),
                    temp_id_mapping: HashMap::from([(
                        mock_item_2.id.clone(),
                        mock_item_2_updated.id.clone(),
                    )]),
                    user: Some(User {
                        full_name: "Drew".to_string(),
                        inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                    }),
                },
            )
            .await;
        let server_url = mock_server.uri();

        let mut cmd = assert_cmd::Command::cargo_bin("tod")?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("sync");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Syncing"))
            .code(0);

        // check that the sync data file was updated with the correct content
        let sync_file = mock_data_dir.join("sync.json");
        let file_contents = fs::read_to_string(sync_file)?;
        let sync_data: Model = serde_json::from_str(&file_contents)?;

        assert_eq!(sync_data.sync_token, "NEW_MOCK_SYNC_TOKEN");
        assert_eq!(sync_data.items.len(), 3);
        assert_eq!(sync_data.items[0].id, mock_item_1.id);
        assert_eq!(sync_data.items[1].id, mock_item_2_updated.id);
        assert_eq!(sync_data.items[2].id, mock_item_3.id);

        // check that a commands list in the data file is now empty
        assert_eq!(sync_data.commands.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn incremental_sync_send_new_todo() -> Result<()> {
        // mock data
        let command_uuid = Uuid::new_v4();
        let mock_item_1 = Item::new("Todo One!", "MOCK_INBOX_PROJECT_ID");
        let mock_item_2 = Item::new("Todo Two!", "MOCK_INBOX_PROJECT_ID");
        let mock_item_2_updated = Item::new("Todo Two!", "MOCK_INBOX_PROJECT_ID");

        // create mock `sync.json` and `commands.json`
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_token = \"MOCK_API_TOKEN\"")?
            .mock_file_contents(
                "sync.json",
                serde_json::to_string_pretty(&Model {
                    sync_token: String::from("MOCK_SYNC_TOKEN"),
                    user: User {
                        full_name: "Drew".to_string(),
                        inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                    },
                    items: vec![mock_item_1.clone(), mock_item_2.clone()],
                    commands: vec![command::Command {
                        request_type: "item_add".to_owned(),
                        temp_id: Some(mock_item_2.id.clone()),
                        uuid: command_uuid,
                        args: Args::AddItemCommandArgs(AddItemArgs {
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo Two!".to_string(),
                        }),
                    }],
                })?,
            )?;
        let mock_data_dir = mock_fs.path();

        // set up mock server
        let mock_server = ApiMockBuilder::new()
            .await
            .mock_response(
                "sync",
                |request: Request| request.sync_token == "MOCK_SYNC_TOKEN",
                Response {
                    full_sync: false,
                    items: vec![],
                    sync_status: Some(HashMap::from([(command_uuid, Status::Ok)])),
                    sync_token: String::from("NEW_MOCK_SYNC_TOKEN"),
                    temp_id_mapping: HashMap::from([(
                        mock_item_2.id.clone(),
                        mock_item_2_updated.id.clone(),
                    )]),
                    user: None,
                },
            )
            .await;
        let server_url = mock_server.uri();

        let mut cmd = assert_cmd::Command::cargo_bin("tod")?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("sync").arg("--incremental");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("Syncing"))
            .code(0);

        // check that the sync data file was updated with the correct content
        let sync_file = mock_data_dir.join("sync.json");
        let file_contents = fs::read_to_string(sync_file)?;
        let sync_data: Model = serde_json::from_str(&file_contents)?;

        assert_eq!(sync_data.sync_token, "NEW_MOCK_SYNC_TOKEN");
        assert_eq!(sync_data.items.len(), 2);
        assert_eq!(sync_data.items[0].id, mock_item_1.id);
        assert_eq!(sync_data.items[1].id, mock_item_2_updated.id);

        // check that a commands list in the data file is now empty
        assert_eq!(sync_data.commands.len(), 0);

        Ok(())
    }
}
