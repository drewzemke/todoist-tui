#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
pub mod api_token_tests {
    use anyhow::Result;
    use std::fs;

    use crate::test_utils::FsMockBuilder;

    #[test]
    fn print_error_when_api_token_missing() -> Result<(), Box<dyn std::error::Error>> {
        // no need to mock the server or file dir, but still going to use fakes for both
        // to prevent accidental calls to the real api or filesystem
        let mock_fs = FsMockBuilder::new()?;
        let mock_data_dir = mock_fs.path();
        let server_url = "fake/server/url";

        // run the thing
        let mut cmd = assert_cmd::Command::cargo_bin("todoist-tui")
            .map_err(|err| format!("Could not run app using 'assert_cmd': {err:?}"))?;
        cmd.arg("--local-dir-override").arg(mock_data_dir);
        cmd.arg("--sync-url-override").arg(server_url);
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
        let mut cmd = assert_cmd::Command::cargo_bin("todoist-tui")
            .map_err(|err| format!("Could not run app using 'assert_cmd': {err:?}"))?;
        cmd.arg("--local-dir-override").arg(mock_data_dir);
        cmd.arg("--sync-url-override").arg(server_url);
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
}
