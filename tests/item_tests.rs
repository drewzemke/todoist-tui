#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
pub mod test_utils;

#[cfg(test)]
pub mod item_tests {
    use anyhow::Result;
    use std::fs;
    use tod::model::{item::Item, user::User, Model};

    use crate::test_utils::FsMockBuilder;

    #[test]
    fn get_inbox_items_from_local() -> Result<()> {
        // mock data
        let mock_item_1 = Item::new("Todo One!", "MOCK_INBOX_PROJECT_ID");
        let mock_item_2 = Item::new("Todo Two!", "MOCK_INBOX_PROJECT_ID");

        // mock `sync.json`
        let mock_fs = FsMockBuilder::new()?.mock_file_contents(
            "sync.json",
            serde_json::to_string_pretty(&Model {
                sync_token: String::from("MOCK_SYNC_TOKEN"),
                user: User {
                    full_name: "Drew".to_string(),
                    inbox_project_id: "MOCK_INBOX_PROJECT_ID".into(),
                },
                items: vec![mock_item_1, mock_item_2],
                ..Default::default()
            })?,
        )?;
        let mock_data_dir = mock_fs.path();

        // no need to mock the server, but still going to use a fake url to prevent
        // accidental calls to the real api
        let server_url = "fake/server/url";

        let mut cmd = assert_cmd::Command::cargo_bin("tod")?;
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
        // mock data
        let mock_item_1 = Item::new("Todo One!", "MOCK_INBOX_PROJECT_ID");
        let mock_item_2 = Item::new("Todo Two!", "MOCK_INBOX_PROJECT_ID");

        // create mock and `sync.json`
        let mock_fs = FsMockBuilder::new()?.mock_file_contents(
            "sync.json",
            serde_json::to_string_pretty(&Model {
                sync_token: String::from("MOCK_SYNC_TOKEN"),
                user: User {
                    full_name: "Drew".to_string(),
                    inbox_project_id: "MOCK_INBOX_PROJECT_ID".into(),
                },
                items: vec![mock_item_1, mock_item_2],
                ..Default::default()
            })?,
        )?;
        let mock_data_dir = mock_fs.path();

        // no need to mock the server, but still going to use a fake url to prevent
        // accidental calls to the real api
        let server_url = "fake/server/url";

        let mut cmd = assert_cmd::Command::cargo_bin("tod")?;
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
        // mock data
        let mock_item_1 = Item::new("Todo One!", "MOCK_INBOX_PROJECT_ID");
        let mock_item_2 = Item::new("Todo Two!", "MOCK_INBOX_PROJECT_ID");

        // create mock and `sync.json`
        let mock_fs = FsMockBuilder::new()?.mock_file_contents(
            "sync.json",
            serde_json::to_string_pretty(&Model {
                sync_token: String::from("MOCK_SYNC_TOKEN"),
                user: User {
                    full_name: "Drew".to_string(),
                    inbox_project_id: "MOCK_INBOX_PROJECT_ID".into(),
                },
                items: vec![mock_item_1, mock_item_2],
                ..Default::default()
            })?,
        )?;
        let mock_data_dir = mock_fs.path();

        // no need to mock the server, but still going to use a fake url to prevent
        // accidental calls to the real api
        let server_url = "fake/server/url";

        let mut cmd = assert_cmd::Command::cargo_bin("tod")?;
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
        let mut cmd = assert_cmd::Command::cargo_bin("tod")?;
        cmd.arg("--local-dir").arg(mock_data_dir);
        cmd.arg("--sync-url").arg(server_url);
        cmd.arg("list");

        // check output again
        cmd.assert()
            .stdout(predicates::str::contains("[1] Todo Two!"))
            .code(0);

        Ok(())
    }
}
