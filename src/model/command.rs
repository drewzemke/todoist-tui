use super::{due_date::Due, item, project};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    // TODO: enumify this
    #[serde(rename = "type")]
    pub request_type: String,
    pub uuid: Uuid,
    pub temp_id: Option<String>,
    pub args: Args,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Args {
    AddItemCommandArgs(AddItemArgs),
    CompleteItemCommandArgs(CompleteItemArgs),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AddItemArgs {
    pub project_id: project::Id,
    pub content: String,
    pub due: Option<Due>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CompleteItemArgs {
    pub id: item::Id,
    // TODO:
    // pub completed_date: ????,
}
