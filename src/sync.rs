use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod client;

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub sync_token: String,
    pub items: Vec<Item>,
    pub user: User,
    pub commands: Vec<Command>,
}

impl Model {
    /// # Errors
    ///
    /// Returns an error if an item with the given id is not found.
    pub fn complete_item(&mut self, item_id: &str) -> Result<&Item> {
        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == item_id)
            .ok_or(anyhow!("Could not find item to complete"))?;
        item.mark_complete();
        Ok(item)
    }

    #[must_use]
    pub fn get_inbox_items(&self) -> Vec<&Item> {
        // get the items with the correct id
        let inbox_id = &self.user.inbox_project_id;
        self.items
            .iter()
            .filter(|item| item.project_id == *inbox_id && !item.checked)
            .collect()
    }
impl TryFrom<Response> for Model {
    type Error = anyhow::Error;

    fn try_from(response: Response) -> std::result::Result<Self, Self::Error> {
        let user = response.user.ok_or(anyhow!(
            "Could not parse a response into a Model because the `user` field was missing"
        ))?;
        Ok(Model {
            sync_token: response.sync_token,
            items: response.items,
            user,

impl Default for Model {
    fn default() -> Self {
        Model {
            sync_token: "*".to_string(),
            items: vec![],
            user: User::default(),
            commands: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub sync_token: String,

    #[serde(default)]
    pub items: Vec<Item>,

    pub user: Option<User>,

    pub full_sync: bool,

    // TODO: make value type more specific?
    pub sync_status: Option<HashMap<String, String>>,

    pub temp_id_mapping: HashMap<Uuid, String>,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub commands: Vec<Command>,
    pub resource_types: Vec<String>,
    pub sync_token: String,
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
    pub checked: bool,
}

impl Item {
    pub fn mark_complete(&mut self) {
        self.checked = true;
    }
}
