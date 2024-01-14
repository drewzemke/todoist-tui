use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::model::project::{Id, Project};

pub struct ProjectTree<'a> {
    items: Vec<TreeItem<'a, Id>>,
    state: TreeState<Id>,
}

impl<'a> ProjectTree<'a> {
    pub fn new(projects: &'_ [Project]) -> Self {
        let items = Self::build_tree(projects, None);
        let mut state = TreeState::default();
        for project in projects {
            if !project.collapsed {
                state.open(vec![project.id.clone()]);
            }
        }

        // Select the first project in the list.
        // TODO: Add a check that this is actually the Inbox project?
        //       Or persist which project was selected on last close?
        if let Some(project) = projects.first() {
            state.select(vec![project.id.clone()]);
        }

        Self { items, state }
    }

    fn build_tree<'b>(projects: &'_ [Project], parent_id: Option<&Id>) -> Vec<TreeItem<'b, Id>> {
        projects
            .iter()
            .filter_map(|project| {
                if project.parent_id.as_ref() == parent_id {
                    // TODO : sort by `project.child_order`
                    let children = Self::build_tree(projects, Some(&project.id));
                    Some(
                        TreeItem::new(project.id.clone(), project.name.clone(), children)
                            .expect("Project ids must be unique"),
                    )
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn selected_project(&self, projects: &'a [Project]) -> Option<&'a Project> {
        self.state
            .selected()
            .last()
            .and_then(|project_id| projects.iter().find(|project| project.id == *project_id))
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('\n' | ' ') => self.state.toggle_selected(),
            KeyCode::Left => self.state.key_left(),
            KeyCode::Right => self.state.key_right(),
            KeyCode::Down => self.state.key_down(&self.items),
            KeyCode::Up => self.state.key_up(&self.items),
            _ => {}
        }
    }

    /// Renders the app state into a terminal frame.
    ///
    /// # Panics
    /// If the model contains projects with duplicate ids
    pub fn tree(&self, focused: bool) -> Tree<'a, Id> {
        Tree::new(self.items.clone())
            .expect("Project ids must be unique")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Projects")
                    .border_style(Style::default().fg(if focused {
                        Color::Yellow
                    } else {
                        Color::Gray
                    })),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
    }

    pub fn state_mut(&mut self) -> &mut TreeState<Id> {
        &mut self.state
    }
}
