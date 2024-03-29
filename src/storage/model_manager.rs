use super::file_manager::FileManager;
use crate::model::Model;
use anyhow::{Context, Result};

const MODEL_FILE_NAME: &str = "sync.json";

pub struct ModelManager<'a> {
    file_manager: &'a FileManager,
}

impl<'a> ModelManager<'a> {
    #[must_use]
    pub fn new(file_manager: &'a FileManager) -> Self {
        Self { file_manager }
    }

    /// # Errors
    ///
    /// Returns an error if the data file cannot be found or read, or if the file
    /// can be read but isn't in the correct format.
    pub fn read_model(&self) -> Result<Model> {
        if self.file_manager.has_data_file(MODEL_FILE_NAME.into()) {
            let file = self
                .file_manager
                .read_data(MODEL_FILE_NAME.into())
                .context("Could not read from the app's data storage.")?;
            let model = serde_json::from_str(&file)
                .with_context(|| format!("Could not parse model file '{MODEL_FILE_NAME}'"))?;
            Ok(model)
        } else {
            Ok(Model::default())
        }
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
}
