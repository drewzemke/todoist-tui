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
pub struct Command {
    #[serde(rename = "type")]
    pub request_type: String,
    pub uuid: Uuid,
    pub temp_id: Option<Uuid>,
    pub args: CommandArgs,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CommandArgs {
    AddItemCommandArgs(AddItemCommandArgs),
    CompleteItemCommandArgs(CompleteItemCommandArgs),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddItemCommandArgs {
    pub project_id: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompleteItemCommandArgs {
    pub id: String,
    // TODO:
    // pub completed_date: ????,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: String,
    pub project_id: String,
    pub content: String,
}
