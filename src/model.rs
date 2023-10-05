use self::{
    command::{AddItemArgs, Args, Command, CompleteItemArgs},
    item::Item,
    project::Project,
    user::User,
};
use crate::sync::{Response, Status};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod command;
pub mod item;
pub mod project;
pub mod user;

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub sync_token: String,
    pub items: Vec<Item>,
    pub projects: Vec<Project>,
    pub user: User,
    pub commands: Vec<Command>,
}

impl Model {
    pub fn add_item(&mut self, item: &str, project_id: &str) {
        let new_item = Item::new(item, project_id);

        self.commands.push(command::Command {
            request_type: "item_add".to_string(),
            temp_id: Some(new_item.id.to_string()),
            uuid: Uuid::new_v4(),
            args: Args::AddItemCommandArgs(AddItemArgs {
                project_id: project_id.to_string(),
                content: item.to_string(),
            }),
        });
        self.items.push(new_item);
    }

    pub fn add_item_to_inbox(&mut self, item: &str) {
        let project_id = self.user.inbox_project_id.clone();
        self.add_item(item, &project_id);
    }

    /// Marks an item as complete (or uncomplete) and creates (removes) a corresponding command
    ///
    /// # Note
    /// This no-ops if an item with the given id does not exist, so check before calling.
    pub fn mark_item(&mut self, item_id: &str, complete: bool) {
        let item = self.items.iter_mut().find(|item| item.id == item_id);

        // If nothing was found, just return
        let Some(item) = item else { return };

        item.mark_complete(complete);

        if complete {
            // Add a new command
            self.commands.push(Command {
                request_type: "item_complete".to_owned(),
                temp_id: None,
                uuid: Uuid::new_v4(),
                args: Args::CompleteItemCommandArgs(CompleteItemArgs {
                    id: item.id.clone(),
                }),
            });
        } else {
            // If there was a pending command to mark this item completed, remove it
            let cmd_index = self.commands.iter().position(|command| {
                if let Args::CompleteItemCommandArgs(CompleteItemArgs { ref id }) = command.args {
                    id == &item.id
                } else {
                    false
                }
            });
            if let Some(cmd_index) = cmd_index {
                self.commands.remove(cmd_index);
            }
        }
    }

    // TODO: test
    #[must_use]
    pub fn get_inbox_items(&self, filter_complete: bool) -> Vec<&Item> {
        // get the items with the correct id
        let inbox_id = &self.user.inbox_project_id;
        self.items
            .iter()
            .filter(|item| item.project_id == *inbox_id && (!filter_complete || !item.checked))
            .collect()
    }

    #[must_use]
    pub fn get_items_in_project(&self, project_id: &str) -> Vec<&Item> {
        self.items
            .iter()
            .filter(|item| item.project_id == *project_id)
            .collect()
    }

    // TODO: test
    #[must_use]
    pub fn projects(&self) -> Vec<&Project> {
        self.projects.iter().collect()
    }

    // TODO: test
    pub fn update(&mut self, response: Response) {
        self.sync_token = response.sync_token;

        if let Some(user) = response.user {
            self.user = user;
        }

        // replace the list of projects with the list from the response.
        // NOTE: this will need to be changed once we introduce commands that
        // modify projects
        self.projects = response.projects;

        if response.full_sync {
            // if this was a full sync, just replace the set of items
            self.items = response.items;
        } else {
            // if not, use the id mapping from the response to update the ids of the existing items
            response
                .temp_id_mapping
                .iter()
                .for_each(|(temp_id, real_id)| {
                    // HACK: should we do something else if we don't find a match?
                    if let Some(matching_item) =
                        self.items.iter_mut().find(|item| item.id == *temp_id)
                    {
                        matching_item.id = real_id.clone();
                    }
                });
        }

        // update the command list by removing the commands that succeeded
        if let Some(ref status_map) = response.sync_status {
            self.commands.retain(|command| {
                !status_map
                    .get(&command.uuid)
                    .is_some_and(|status| *status == Status::Ok)
            });
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        let inbox = Project::new("Inbox");
        let user = User {
            inbox_project_id: inbox.id.clone(),
            ..Default::default()
        };
        Model {
            sync_token: "*".to_string(),
            items: vec![],
            projects: vec![inbox],
            user,
            commands: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_item_to_inbox() {
        let mut model = Model::default();
        model.user.inbox_project_id = "INBOX_ID".to_string();
        model.add_item_to_inbox("New item!");

        assert_eq!(model.items[0].project_id, "INBOX_ID");
        assert_eq!(model.items[0].content, "New item!");
        assert_eq!(model.commands[0].request_type, "item_add");
        assert_eq!(
            model.commands[0].args,
            Args::AddItemCommandArgs(AddItemArgs {
                project_id: "INBOX_ID".to_string(),
                content: "New item!".to_string()
            })
        );
    }

    #[test]
    fn mark_item_completed() {
        let mut model = Model::default();
        let item = Item::new("Item!", "INBOX_ID");
        let item_id = item.id.clone();
        model.items.push(item);
        model.mark_item(&item_id, true);

        assert!(model.items[0].checked);
        assert_eq!(model.commands[0].request_type, "item_complete");
        assert_eq!(
            model.commands[0].args,
            Args::CompleteItemCommandArgs(CompleteItemArgs { id: item_id })
        );
    }
}
