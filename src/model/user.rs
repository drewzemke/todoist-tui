use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct User {
    pub full_name: String,
    pub inbox_project_id: String,
}
