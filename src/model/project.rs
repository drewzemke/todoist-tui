use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(clippy::module_name_repetitions)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: Id,
    pub name: String,
}

impl Project {
    /// Create a new project with a given name. Generates a random uuid as an id.
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: Uuid::new_v4().to_string().into(),
            name: name.into(),
        }
    }
}
