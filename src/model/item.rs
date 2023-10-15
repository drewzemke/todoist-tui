use std::fmt::Display;

use super::project;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    // TODO: newtype
    pub id: String,
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
            id: Uuid::new_v4().to_string(),
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum DueDate {
    Date(NaiveDate),
    DateTime(NaiveDateTime),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Due {
    pub date: DueDate,
}

impl Display for Due {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.date {
            DueDate::Date(date) => date.fmt(f),
            DueDate::DateTime(datetime) => datetime.fmt(f),
        }
    }
}
