use std::fmt::Display;

use super::{due_date::Due, project, section};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An id for an item, which is really just `String`.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
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
    pub parent_id: Option<Id>,
    pub child_order: i32,
    pub section_id: Option<section::Id>,
    pub collapsed: bool,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string().into(),
            project_id: Uuid::new_v4().to_string().into(),
            content: String::new(),
            checked: false,
            due: None,
            parent_id: None,
            child_order: 0,
            section_id: None,
            collapsed: false,
        }
    }
}

impl Item {
    /// Creates a new item with the given content and project id. Creates a random ID using `UUIDv4`.
    pub fn new<S, P>(content: S, project_id: P) -> Self
    where
        S: Into<String>,
        P: Into<project::Id>,
    {
        Self {
            content: content.into(),
            project_id: project_id.into(),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.mark_complete(checked);
        self
    }

    #[must_use]
    pub fn due(mut self, due: Option<Due>) -> Self {
        self.due = due;
        self
    }

    #[must_use]
    pub fn parent_id(mut self, parent_id: impl Into<Id>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    #[must_use]
    pub fn child_order(mut self, child_order: i32) -> Self {
        self.child_order = child_order;
        self
    }

    #[must_use]
    pub fn section_id(mut self, section_id: impl Into<section::Id>) -> Self {
        self.section_id = Some(section_id.into());
        self
    }

    pub fn mark_complete(&mut self, complete: bool) {
        self.checked = complete;
    }
}
