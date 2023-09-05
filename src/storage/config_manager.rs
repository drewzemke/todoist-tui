use super::file_manager::FileManager;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const AUTH_FILE_NAME: &str = "client_auth.toml";

#[derive(Serialize, Deserialize)]
pub struct Auth {
    pub api_token: String,
}

pub struct ConfigManager<'a> {
    file_manager: &'a FileManager,
}

impl<'a> ConfigManager<'a> {
    #[must_use]
    pub fn new(file_manager: &'a FileManager) -> Self {
        Self { file_manager }
    }

    /// # Errors
    ///
    /// Returns an error if the config file cannot be found or read, or if the file
    /// can be read but isn't in the correct format.
    pub fn read_auth_config(&self) -> Result<Auth> {
        let file = self
            .file_manager
            .read_data(AUTH_FILE_NAME.into())
            .context(concat!(
                "Could not find an API token. ",
                "Go to https://todoist.com/app/settings/integrations/developer to get yours, ",
                "then re-run with command 'set-token <TOKEN>'."
            ))?;
        let config: Auth = toml::from_str(file.as_str())
            .with_context(|| format!("Could not parse config file '{AUTH_FILE_NAME}'"))?;
        Ok(config)
    }

    /// # Errors
    ///
    /// Returns an error if something goes wrong while writing to the file.
    pub fn write_auth_config(&self, config: &Auth) -> Result<()> {
        let contents = toml::to_string_pretty(config)?;
        self.file_manager
            .write_data(AUTH_FILE_NAME.into(), &contents)?;
        Ok(())
    }
}
