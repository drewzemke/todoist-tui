use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An id for a project, which is really just `String`.
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

/// Represents a todoist project.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: Id,
    pub name: String,
    pub parent_id: Option<Id>,
    pub child_order: i32,
    pub collapsed: bool,
}

impl Project {
    /// Create a new project with a given name. Generates a random uuid as an id.
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Sets a parent id for a project.
    /// This consumes the project and returns a new one.
    #[must_use]
    pub fn parent_id(mut self, parent_id: impl Into<Id>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Sets a child order for a project, which is the order in
    /// which a project appears relative to its siblings.
    /// This consumes the project and returns a new one.
    #[must_use]
    pub fn child_order(mut self, child_order: i32) -> Self {
        self.child_order = child_order;
        self
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string().into(),
            name: String::new(),
            parent_id: None,
            child_order: 0,
            collapsed: false,
        }
    }
}
