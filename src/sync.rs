use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::model::{command::Command, item::Item, user::User};

pub mod client;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub sync_token: String,

    #[serde(default)]
    pub items: Vec<Item>,

    pub user: Option<User>,

    pub full_sync: bool,

    pub sync_status: Option<HashMap<Uuid, Status>>,

    pub temp_id_mapping: HashMap<Uuid, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub commands: Vec<Command>,
    // TODO: stronger typing
    pub resource_types: Vec<String>,
    pub sync_token: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Status {
    #[serde(rename = "ok")]
    Ok,
    #[serde(untagged)]
    Error(StatusError),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StatusError {
    #[serde(rename = "error_code")]
    pub code: u32,
    #[serde(rename = "error")]
    pub message: String,
}
