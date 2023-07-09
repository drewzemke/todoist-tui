use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub full_sync: bool,

    // TODO: make value type more specific?
    pub sync_status: Option<HashMap<String, String>>,

    pub sync_token: String,
    pub temp_id_mapping: HashMap<Uuid, String>,
    pub user: Option<User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub full_name: String,
    pub inbox_project_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddItemRequest {
    pub sync_token: String,
    pub resource_types: Vec<String>,
    pub commands: Vec<AddItemCommand>,
}

// they're the same, fine for now
#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserRequest {
    pub sync_token: String,
    pub resource_types: Vec<String>,
    pub commands: Vec<AddItemCommand>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddItemCommand {
    #[serde(rename = "type")]
    pub request_type: String,

    pub temp_id: Uuid,
    pub uuid: Uuid,
    pub args: AddItemRequestArgs,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddItemRequestArgs {
    pub project_id: String,
    pub content: String,
}
