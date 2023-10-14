use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::model::{command::Command, item::Item, project::Project, user::User};

pub mod client;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Response {
    pub sync_token: String,

    #[serde(default)]
    pub projects: Vec<Project>,

    #[serde(default)]
    pub items: Vec<Item>,

    pub user: Option<User>,

    pub full_sync: bool,

    pub sync_status: Option<HashMap<Uuid, Status>>,

    pub temp_id_mapping: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub commands: Vec<Command>,
    pub resource_types: Vec<ResourceType>,
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    #[serde(rename = "items")]
    Items,
    #[serde(rename = "projects")]
    Projects,
    #[serde(rename = "user")]
    User,
}

impl ResourceType {
    /// Returns all of the resource types that should be requested in a full sync
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![Self::Items, Self::Projects, Self::User]
    }
}
