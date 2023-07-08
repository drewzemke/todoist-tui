use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub full_sync: bool,

    // TODO: make value type more specific?
    pub sync_status: HashMap<String, String>,

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
pub struct AddItemSyncRequest {
    pub sync_token: String,
    pub resource_types: Vec<String>,
    pub commands: Vec<AddItemSyncCommand>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddItemSyncCommand {
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
