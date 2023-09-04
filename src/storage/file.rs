use anyhow::{bail, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

const APP_DATA_DIR_NAME: &str = "tuido";

pub struct Manager {
    data_dir: PathBuf,
}

impl Manager {
    /// # Errors
    ///
    /// Returns an error if the local data directory cannot be found.
    pub fn init(data_dir_override: Option<String>) -> Result<Self> {
        let data_dir = if let Some(dir) = data_dir_override {
            PathBuf::from(dir)
        } else if let Some(dir) = dirs::data_local_dir() {
            dir.join(APP_DATA_DIR_NAME)
        } else {
            bail!("Could not find local data directory.");
        };

        if !data_dir.exists() {
            fs::create_dir(&data_dir)?;
        }

        Ok(Self { data_dir })
    }

    /// # Errors
    ///
    /// Returns an error if the file does not exist, cannot be opened, or if
    /// an error occurs while reading.
    pub fn read_data(&self, path_from_data_dir: PathBuf) -> Result<String> {
        let file_path = Path::new(&self.data_dir).join(path_from_data_dir);
        let file = fs::read_to_string(file_path)?;
        Ok(file)
    }

    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or if an error occurs while writing.
    pub fn write_data(&self, path_from_data_dir: PathBuf, data: &str) -> Result<()> {
        let file_path = Path::new(&self.data_dir).join(path_from_data_dir);
        fs::write(file_path, data)?;
        Ok(())
    }

    #[must_use]
    pub fn has_data_file(&self, path_from_data_dir: PathBuf) -> bool {
        let file_path = Path::new(&self.data_dir).join(path_from_data_dir);
        file_path.exists()
    }
}
