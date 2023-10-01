use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
}

impl Project {
    /// Create a new project with a given name. Generates a random uuid as an id.
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
        }
    }
}
