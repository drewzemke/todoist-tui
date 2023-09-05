use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub full_name: String,
    pub inbox_project_id: String,
}

impl Default for User {
    fn default() -> Self {
        User {
            full_name: "First Last".to_string(),
            inbox_project_id: String::new(),
        }
    }
}
