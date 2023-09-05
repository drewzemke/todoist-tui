#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
// HACK: turn this off at some point
#![allow(clippy::too_many_lines)]
pub mod test_utils;

#[cfg(test)]
pub mod sync {
    use anyhow::Result;
    use assert_cmd::Command;
    use std::{collections::HashMap, fs};
    use todoist::{
        model::{
            command::{self, AddItemArgs, Args},
            item::Item,
            user::User,
            Model,
        },
        sync::{Request, ResourceType, Response, Status},
    };
    use uuid::Uuid;

    use crate::test_utils::{ApiMockBuilder, FsMockBuilder};

    #[tokio::test]
    async fn full_sync_and_store_data() -> Result<()> {
        // create mock `client_auth.toml`
        let mock_fs = FsMockBuilder::new()?
            .mock_file_contents("client_auth.toml", "api_token = \"MOCK_API_TOKEN\"")?;
        let mock_data_dir = mock_fs.path();

        // set up mock server
        let mock_server = ApiMockBuilder::new()
            .await
            .mock_response(
                "sync",
                |request: Request| {
                    request.sync_token == "*"
                        && request
                            .resource_types
                            .get(0)
                            .is_some_and(|s| *s == ResourceType::All)
                },
                Response {
                    full_sync: true,
                    items: vec![
                        Item {
                            id: "MOCK_ITEM_ID_1".to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo One!".to_string(),
                            checked: false,
                        },
                        Item {
                            id: "MOCK_ITEM_ID_2".to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo Two!".to_string(),
                            checked: false,
                        },
                    ],
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
        let mut cmd = Command::cargo_bin("todoist")?;
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

    #[test]
    fn get_inbox_items_from_local() -> Result<()> {
        // create mock `sync.json`
        let mock_fs = FsMockBuilder::new()?.mock_file_contents(
            "sync.json",
            serde_json::to_string_pretty(&Model {
                sync_token: String::from("MOCK_SYNC_TOKEN"),
                user: User {
                    full_name: "Drew".to_string(),
                    inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                },
                items: vec![
                    Item {
                        id: "MOCK_ITEM_ID_1".to_string(),
                        project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                        content: "Todo One!".to_string(),
                        checked: false,
                    },
                    Item {
                        id: "MOCK_ITEM_ID_2".to_string(),
                        project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                        content: "Todo Two!".to_string(),
                        checked: false,
                    },
                ],
                commands: vec![],
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
            .stdout(predicates::str::contains("[1] Todo One!"))
            .stdout(predicates::str::contains("[2] Todo Two!"))
            .code(0);

        Ok(())
    }

    #[test]
    fn add_todo_to_local_no_sync() -> Result<()> {
        // create mock and `sync.json`
        let mock_fs = FsMockBuilder::new()?.mock_file_contents(
            "sync.json",
            serde_json::to_string_pretty(&Model {
                sync_token: String::from("MOCK_SYNC_TOKEN"),
                user: User {
                    full_name: "Drew".to_string(),
                    inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                },
                items: vec![
                    Item {
                        id: "MOCK_ITEM_ID_1".to_string(),
                        project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                        content: "Todo One!".to_string(),
                        checked: false,
                    },
                    Item {
                        id: "MOCK_ITEM_ID_2".to_string(),
                        project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                        content: "Todo Two!".to_string(),
                        checked: false,
                    },
                ],
                commands: vec![],
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
        cmd.arg("--no-sync");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("'new todo!' added"))
            .code(0);

        // check that a command was created in the data file with the correct content
        let data_file = mock_data_dir.join("sync.json");
        assert!(data_file.exists());

        let file_contents = fs::read_to_string(data_file)?;
        let model: Model = serde_json::from_str(&file_contents)?;
        assert_eq!(model.commands.len(), 1);
        assert_eq!(model.commands[0].request_type, "item_add");

        Ok(())
    }

    #[test]
    fn complete_todo_no_sync() -> Result<()> {
        // create mock and `sync.json`
        let mock_fs = FsMockBuilder::new()?.mock_file_contents(
            "sync.json",
            serde_json::to_string_pretty(&Model {
                sync_token: String::from("MOCK_SYNC_TOKEN"),
                user: User {
                    full_name: "Drew".to_string(),
                    inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                },
                items: vec![
                    Item {
                        id: "MOCK_ITEM_ID_1".to_string(),
                        project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                        content: "Todo One!".to_string(),
                        checked: false,
                    },
                    Item {
                        id: "MOCK_ITEM_ID_2".to_string(),
                        project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                        content: "Todo Two!".to_string(),
                        checked: false,
                    },
                ],
                commands: vec![],
            })?,
        )?;
        let mock_data_dir = mock_fs.path();

        // no need to mock the server, but still going to use a fake url to prevent
        // accidental calls to the real api
        let server_url = "fake/server/url";

        let mut cmd = assert_cmd::Command::cargo_bin("todoist")?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("complete").arg("1");
        cmd.arg("--no-sync");

        // check output
        cmd.assert()
            .stdout(predicates::str::contains("'Todo One!' marked complete"))
            .code(0);

        // check that a command was created in the data file with the correct content
        let data_file = mock_data_dir.join("sync.json");
        assert!(data_file.exists());

        let file_contents = fs::read_to_string(data_file)?;
        let model: Model = serde_json::from_str(&file_contents)?;
        assert_eq!(model.commands.len(), 1);
        assert_eq!(model.commands[0].request_type, "item_complete");

        // the completed todo should no longer appear when running 'list'
        let mut cmd = assert_cmd::Command::cargo_bin("todoist")?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("list");

        // check output again
        cmd.assert()
            .stdout(predicates::str::contains("[1] Todo Two!"))
            .code(0);

        Ok(())
    }

    #[tokio::test]
    async fn full_sync_send_new_todo() -> Result<()> {
        let new_item_temp_id = Uuid::new_v4();
        let command_uuid = Uuid::new_v4();

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
                    items: vec![
                        Item {
                            id: "MOCK_ITEM_ID_1".to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo One!".to_string(),
                            checked: false,
                        },
                        Item {
                            id: new_item_temp_id.to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo Two!".to_string(),
                            checked: false,
                        },
                    ],
                    commands: vec![command::Command {
                        request_type: "item_add".to_owned(),
                        temp_id: Some(new_item_temp_id),
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
                |request: Request| {
                    request.sync_token == "*"
                        && request
                            .resource_types
                            .get(0)
                            .is_some_and(|s| *s == ResourceType::All)
                },
                Response {
                    full_sync: true,
                    items: vec![
                        Item {
                            id: "MOCK_ITEM_ID_1".to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo One!".to_string(),
                            checked: false,
                        },
                        Item {
                            id: "MOCK_ITEM_ID_2_NEW".to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo Two!".to_string(),
                            checked: false,
                        },
                        Item {
                            id: "MOCK_ITEM_ID_3".to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo Three!".to_string(),
                            checked: false,
                        },
                    ],
                    sync_status: Some(HashMap::from([(command_uuid, Status::Ok)])),
                    sync_token: String::from("NEW_MOCK_SYNC_TOKEN"),
                    temp_id_mapping: HashMap::from([(
                        new_item_temp_id,
                        "MOCK_ITEM_ID_2_NEW".to_string(),
                    )]),
                    user: Some(User {
                        full_name: "Drew".to_string(),
                        inbox_project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                    }),
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
            .stdout(predicates::str::contains("Syncing"))
            .code(0);

        // check that the sync data file was updated with the correct content
        let sync_file = mock_data_dir.join("sync.json");
        let file_contents = fs::read_to_string(sync_file)?;
        let sync_data: Model = serde_json::from_str(&file_contents)?;

        assert_eq!(sync_data.sync_token, "NEW_MOCK_SYNC_TOKEN");
        assert_eq!(sync_data.items.len(), 3);
        assert_eq!(sync_data.items[0].id, "MOCK_ITEM_ID_1");
        assert_eq!(sync_data.items[1].id, "MOCK_ITEM_ID_2_NEW");
        assert_eq!(sync_data.items[2].id, "MOCK_ITEM_ID_3");

        // check that a commands list in the data file is now empty
        assert_eq!(sync_data.commands.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn incremental_sync_send_new_todo() -> Result<()> {
        let new_item_temp_id = Uuid::new_v4();
        let command_uuid = Uuid::new_v4();

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
                    items: vec![
                        Item {
                            id: "MOCK_ITEM_ID_1".to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo One!".to_string(),
                            checked: false,
                        },
                        Item {
                            id: new_item_temp_id.to_string(),
                            project_id: "MOCK_INBOX_PROJECT_ID".to_string(),
                            content: "Todo Two!".to_string(),
                            checked: false,
                        },
                    ],
                    commands: vec![command::Command {
                        request_type: "item_add".to_owned(),
                        temp_id: Some(new_item_temp_id),
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
                |request: Request| {
                    request.sync_token == "MOCK_SYNC_TOKEN"
                        && request
                            .resource_types
                            .get(0)
                            .is_some_and(|s| *s == ResourceType::All)
                },
                Response {
                    full_sync: false,
                    items: vec![],
                    sync_status: Some(HashMap::from([(command_uuid, Status::Ok)])),
                    sync_token: String::from("NEW_MOCK_SYNC_TOKEN"),
                    temp_id_mapping: HashMap::from([(
                        new_item_temp_id,
                        "MOCK_ITEM_ID_2_NEW".to_string(),
                    )]),
                    user: None,
                },
            )
            .await;
        let server_url = mock_server.uri();

        let mut cmd = assert_cmd::Command::cargo_bin("todoist")?;
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
        assert_eq!(sync_data.items[0].id, "MOCK_ITEM_ID_1");
        assert_eq!(sync_data.items[1].id, "MOCK_ITEM_ID_2_NEW");

        // check that a commands list in the data file is now empty
        assert_eq!(sync_data.commands.len(), 0);

        Ok(())
    }
}
