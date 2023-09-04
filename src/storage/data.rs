// TODO: move/rename to 'model/manager.rs'
use super::file::Manager as FileManager;
use crate::sync::{Command, Model};
use anyhow::{Context, Result};

const MODEL_FILE_NAME: &str = "sync.json";
const COMMANDS_FILE_NAME: &str = "commands.json";

pub struct Manager<'a> {
    file_manager: &'a FileManager,
}

impl<'a> Manager<'a> {
    #[must_use]
    pub fn new(file_manager: &'a FileManager) -> Self {
        Self { file_manager }
    }

    /// # Errors
    ///
    /// Returns an error if the data file cannot be found or read, or if the file
    /// can be read but isn't in the correct format.
    pub fn read_model(&self) -> Result<Model> {
        let file = self
            .file_manager
            .read_data(MODEL_FILE_NAME.into())
            .context("Could not read from the app's data storage.")?;
        let model = serde_json::from_str(&file)
            .with_context(|| format!("Could not parse model file '{MODEL_FILE_NAME}'"))?;
        Ok(model)
    }

    /// # Errors
    ///
    /// Returns an error if something goes wrong while writing to the file.
    pub fn write_model(&self, model: &Model) -> Result<()> {
        let contents = serde_json::to_string_pretty(model)?;
        self.file_manager
            .write_data(MODEL_FILE_NAME.into(), &contents)
            .context("Could not write to command storage file.")?;
        Ok(())
    }
    // TODO: combine the two methods below with the above (while also combining Model and Commands?)
    /// # Errors
    ///
    /// Returns an error if the data file cannot be found or read, or if the file
    /// can be read but isn't in the correct format.
    pub fn read_commands(&self) -> Result<Vec<Command>> {
        let file = self
            .file_manager
            .read_data(COMMANDS_FILE_NAME.into())
            .context("Could not read from the app's command storage.")?;
        let commands = serde_json::from_str(&file)
            .with_context(|| format!("Could not parse command file '{COMMANDS_FILE_NAME}'"))?;
        Ok(commands)
    }

    /// # Errors
    ///
    /// Returns an error if something goes wrong while writing to the file.
    pub fn write_commands(&self, commands: &Vec<Command>) -> Result<()> {
        let contents = serde_json::to_string_pretty(commands)?;
        self.file_manager
            .write_data(COMMANDS_FILE_NAME.into(), &contents)
            .context("Could not write to command storage file.")?;
        Ok(())
    }
}
