use super::app_state::{AppState, Mode};
use crate::model::project::{Id as ProjectId, Project};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, StatefulWidget},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

pub struct ProjectsState<'a> {
    tree_items: Vec<TreeItem<'a, ProjectId>>,
    state: TreeState<ProjectId>,
    default_project_id: ProjectId,
}

impl<'a> ProjectsState<'a> {
    //  HACK?
    /// # Panics
    /// If the list of projects is empty.
    pub fn new(projects: &'_ [Project]) -> Self {
        let tree_items = Self::build_tree(projects, None);
        let mut state = TreeState::default();
        for project in projects {
            if !project.collapsed {
                state.open(vec![project.id.clone()]);
            }
        }

        // Select the first project in the list.
        // TODO: Add a check that this is actually the Inbox project?
        //       Or persist which project was selected on last close?
        let first_project = projects
            .first()
            .expect("There should always be at least one project.");
        state.select(vec![first_project.id.clone()]);

        Self {
            tree_items,
            state,
            default_project_id: first_project.id.clone(),
        }
    }

    fn build_tree<'b>(
        projects: &'_ [Project],
        parent_id: Option<&ProjectId>,
    ) -> Vec<TreeItem<'b, ProjectId>> {
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

    pub fn selected(&self) -> ProjectId {
        self.state
            .selected()
            .into_iter()
            .last()
            .unwrap_or(self.default_project_id.clone())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('\n' | ' ') => self.state.toggle_selected(),
            KeyCode::Left => self.state.key_left(),
            KeyCode::Right => self.state.key_right(),
            KeyCode::Down => self.state.key_down(&self.tree_items),
            KeyCode::Up => self.state.key_up(&self.tree_items),
            _ => {}
        }
    }
}

#[derive(Debug, Default)]
pub struct ProjectsPane<'a> {
    marker: std::marker::PhantomData<AppState<'a>>,
}

impl<'a> StatefulWidget for ProjectsPane<'a> {
    type State = AppState<'a>;

    /// Renders the app state into a terminal frame.
    ///
    /// # Panics
    /// If the model contains projects with duplicate ids
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let focused = state.mode == Mode::SelectingProjects;

        let tree = Tree::new(state.projects.tree_items.clone())
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
            );

        tree.render(area, buf, &mut state.projects.state);
    }
}
