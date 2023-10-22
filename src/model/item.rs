use std::fmt::Display;

use super::{due_date::Due, project};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An id for an item, which is really just `String`.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Id(String);

impl From<String> for Id {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<&Id> for Id {
    fn from(value: &Id) -> Self {
        Self(value.0.clone())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: Id,
    pub project_id: project::Id,
    pub content: String,
    pub checked: bool,
    pub due: Option<Due>,
}

impl Item {
    /// Creates a new item with the given content and project id. Creates a random ID using `UUIDv4`.
    pub fn new<S, P>(content: S, project_id: P) -> Self
    where
        S: Into<String>,
        P: Into<project::Id>,
    {
        Self {
            id: Uuid::new_v4().to_string().into(),
            content: content.into(),
            project_id: project_id.into(),
            checked: false,
            due: None,
        }
    }

    // TODO : builder pattern for Item
    // like, Item::new(...).project(...).due(...)

    pub fn mark_complete(&mut self, complete: bool) {
        self.checked = complete;
    }
}
