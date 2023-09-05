use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: String,
    pub project_id: String,
    pub content: String,
    pub checked: bool,
}

impl Item {
    pub fn mark_complete(&mut self) {
        self.checked = true;
    }
}
