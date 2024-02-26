use self::{
    command::{AddItemArgs, Args, Command, CompleteItemArgs},
    due_date::Due,
    item::Item,
    project::Project,
    section::Section,
    user::User,
};
use crate::sync::{Response, Status};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod command;
pub mod due_date;
pub mod item;
pub mod project;
pub mod section;
pub mod user;

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub sync_token: String,
    pub items: Vec<Item>,
    pub projects: Vec<Project>,
    pub sections: Vec<Section>,
    pub user: User,
    pub commands: Vec<Command>,
}

impl Model {
    pub fn add_item(&mut self, item: &str, project_id: project::Id, due_date: Option<Due>) {
        let new_item = Item::new(item, &project_id).due(due_date.clone());

        self.commands.push(command::Command {
            request_type: "item_add".to_string(),
            temp_id: Some(new_item.id.to_string()),
            uuid: Uuid::new_v4(),
            args: Args::AddItemCommandArgs(AddItemArgs {
                project_id,
                content: item.to_string(),
                due: due_date,
            }),
        });
        self.items.push(new_item);
    }

    pub fn add_item_to_inbox(&mut self, item: &str, due_date: Option<Due>) {
        let project_id = self.user.inbox_project_id.clone();
        self.add_item(item, project_id, due_date);
    }

    /// Marks an item as complete (or uncomplete) and creates (removes) a corresponding command
    ///
    /// # Note
    /// This no-ops if an item with the given id does not exist, so check before calling.
    pub fn mark_item(&mut self, item_id: &item::Id, complete: bool) {
        let item = self.items.iter_mut().find(|item| &item.id == item_id);

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
    pub fn items_in_project(&self, project_id: &project::Id) -> Vec<&Item> {
        self.items
            .iter()
            .filter(|item| item.project_id == *project_id)
            .collect()
    }

    #[must_use]
    pub fn sections_and_items_in_project(
        &self,
        project_id: &project::Id,
    ) -> Vec<(Option<&Section>, Vec<&Item>)> {
        let mut sections: Vec<Option<&Section>> = self
            .sections
            .iter()
            .filter(|section| section.project_id == *project_id)
            .map(Some)
            .collect();
        sections.insert(0, None);
        sections.sort_unstable();

        sections
            .into_iter()
            .map(|section| {
                let items_in_section: Vec<_> = self
                    .items
                    .iter()
                    .filter(|item| item.project_id == *project_id)
                    .filter(|item| match (&item.section_id, section) {
                        (Some(id), Some(section)) => section.id == *id,
                        (None, None) => true,
                        _ => false,
                    })
                    .collect();
                (section, items_in_section)
            })
            .collect()
    }

    // TODO: test
    #[must_use]
    pub fn projects(&self) -> Vec<&Project> {
        self.projects.iter().collect()
    }

    /// # Panics
    /// If there is no inbox project in the model.
    #[must_use]
    pub fn inbox_project(&self) -> &Project {
        let inbox_id = &self.user.inbox_project_id;
        self.projects
            .iter()
            .find(|project| project.id == *inbox_id)
            .expect("there should always be an inbox project")
    }

    #[must_use]
    pub fn project_with_id(&self, id: &project::Id) -> Option<&Project> {
        self.projects.iter().find(|project| project.id == *id)
    }

    pub fn update(&mut self, response: Response) {
        self.sync_token = response.sync_token;

        if let Some(user) = response.user {
            self.user = user;
        }

        // replace the list of projects with the list from the response.
        // FIXME: we need a more nuanced algorithm to update the projects.
        // just replacing `self.projects` with `response.projects` is no good
        // because we don't always query all projects
        // so for now, only replace projects if the incoming projects
        // list is nonempty
        if !response.projects.is_empty() {
            self.projects = response.projects;
        }

        // same thing (and same FIXME) with sections
        if !response.sections.is_empty() {
            self.sections = response.sections;
        }

        if response.full_sync {
            // if this was a full sync, just replace the set of items
            self.items = response.items;
        } else {
            // use the id mapping from the response to update the ids of the existing items
            response
                .temp_id_mapping
                .into_iter()
                .for_each(|(temp_id, real_id)| {
                    // HACK: should we do something else if we don't find a match?
                    if let Some(matching_item) = self
                        .items
                        .iter_mut()
                        .find(|item| item.id == temp_id.clone().into())
                    {
                        matching_item.id = real_id.into();
                    }
                });

            // look through the list of items that we received -- add any new items
            // and update any newly checked items
            response.items.into_iter().for_each(|incoming_item| {
                // if the incoming item is checked, see if there's a matching item in the model
                // and remove it
                // if the incoming item is unchecked, do the same, but if nothing is found, add this
                // item to the model
                let matching_index = self
                    .items
                    .iter()
                    .position(|item| item.id == incoming_item.id);

                match matching_index {
                    Some(index) => {
                        if incoming_item.checked {
                            self.items.remove(index);
                        } else {
                            self.items[index] = incoming_item;
                        }
                    }
                    None => self.items.push(incoming_item),
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
            sections: vec![],
            user,
            commands: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn add_item_to_inbox() {
        let mut model = Model::default();
        model.user.inbox_project_id = "INBOX_ID".into();
        model.add_item_to_inbox("New item!", None);

        assert_eq!(model.items[0].project_id, "INBOX_ID".into());
        assert_eq!(model.items[0].content, "New item!");
        assert_eq!(model.commands[0].request_type, "item_add");
        assert_eq!(
            model.commands[0].args,
            Args::AddItemCommandArgs(AddItemArgs {
                project_id: "INBOX_ID".into(),
                content: "New item!".to_string(),
                due: None
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

    #[test]
    fn incremental_update_with_updated_todos() {
        let mut model = Model::default();
        let item1 = Item::new("Item One!", "INBOX_ID");
        let item1_updated = Item {
            checked: true,
            ..item1.clone()
        };

        let item2 = Item::new("Item Two!", "INBOX_ID");
        let item2_updated = Item {
            content: "Item Two with a new title!".into(),
            ..item2.clone()
        };

        model.items.push(item1);
        model.items.push(item2);
        let response = Response {
            items: vec![item1_updated, item2_updated],
            full_sync: false,
            ..Default::default()
        };

        model.update(response);
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].content, "Item Two with a new title!");
    }

    #[test]
    fn incremental_update_with_new_external_todo() {
        let mut model = Model::default();
        let item = Item::new("Item!", "INBOX_ID");

        let response = Response {
            items: vec![item],
            full_sync: false,
            ..Default::default()
        };

        model.update(response);
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].content, "Item!");
    }

    #[test]
    fn incremental_update_after_adding_local_todo() {
        let mut model = Model::default();
        let item = Item::new("Item!", "INBOX_ID");
        let item_updated = Item {
            id: "NEW_ITEM_ID".into(),
            ..item.clone()
        };
        let item_id = item.id.clone();
        model.items.push(item);

        let response = Response {
            items: vec![item_updated],
            full_sync: false,
            temp_id_mapping: HashMap::from([(item_id.to_string(), "NEW_ITEM_ID".into())]),
            ..Default::default()
        };

        model.update(response);
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].content, "Item!");
    }
}
