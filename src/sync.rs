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

    #[serde(default)]
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub full_name: String,
    pub inbox_project_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub sync_token: String,
    pub resource_types: Vec<String>,
    pub commands: Vec<Command>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Command {
    AddItem(AddItemCommand),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddItemCommand {
    #[serde(rename = "type")]
    pub request_type: String,

    pub temp_id: Uuid,
    pub uuid: Uuid,
    pub args: AddItemRequestArgs,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddItemRequestArgs {
    pub project_id: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDataRequest {
    pub project_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDataResponse {
    pub project: Project,
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub project_id: String,
    pub content: String,
}
