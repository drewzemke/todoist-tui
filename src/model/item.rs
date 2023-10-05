use super::project;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: String,
    pub project_id: project::Id,
    pub content: String,
    pub checked: bool,
}

impl Item {
    /// Creates a new item with the given content and project id. Creates a random ID using `UUIDv4`.
    pub fn new<S, P>(content: S, project_id: P) -> Self
    where
        S: Into<String>,
        P: Into<project::Id>,
    {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            project_id: project_id.into(),
            checked: false,
        }
    }

    pub fn mark_complete(&mut self, complete: bool) {
        self.checked = complete;
    }
}
