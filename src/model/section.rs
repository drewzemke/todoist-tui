use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::project;

/// An id for a section, which is really just `String`.
// TODO: this is identical code to two other modules, is there a way to combine?
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

/// Represents a section inside a project.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Section {
    pub id: Id,
    pub name: String,
    pub project_id: project::Id,
    pub section_order: i32,
}

impl Section {
    /// Create a new project with a given name. Generates a random uuid as an id.
    pub fn new<S>(name: S, project_id: impl Into<project::Id>) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            project_id: project_id.into(),
            ..Default::default()
        }
    }

    /// Sets a section order for a section, which describes the order
    /// in which it appears relative to the other sections in a project.
    /// This consumes the section and returns a new one.
    #[must_use]
    pub fn section_order(mut self, section_order: i32) -> Self {
        self.section_order = section_order;
        self
    }
}

impl Default for Section {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string().into(),
            name: String::new(),
            project_id: "".into(),
            section_order: 0,
        }
    }
}
