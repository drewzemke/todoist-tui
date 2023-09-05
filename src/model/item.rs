use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: String,
    pub project_id: String,
    pub content: String,
    pub checked: bool,
}

impl Item {
    /// Creates a new item with the given content and project id. Creates a random ID using `UUIDv4`.
    pub fn new<S>(content: S, project_id: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            project_id: project_id.into(),
            checked: false,
        }
    }

    pub fn mark_complete(&mut self) {
        self.checked = true;
    }
}
