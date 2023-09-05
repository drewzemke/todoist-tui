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

    // TODO: make value type more specific?
    // also key type should probs be a UUID
    pub sync_status: Option<HashMap<String, String>>,

    pub temp_id_mapping: HashMap<Uuid, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub commands: Vec<Command>,
    // TODO: stronger typing
    pub resource_types: Vec<String>,
    pub sync_token: String,
}
