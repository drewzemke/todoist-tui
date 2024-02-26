use crate::{
    model::{
        project::{Id as ProjectId, Project},
        Model,
    },
    tui::app_state::{AppState, Mode},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, StatefulWidget},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

pub struct State {
    /// Records which project's are expanded and which is selected.
    tree: TreeState<ProjectId>,

    /// When a key is pressed, we wait until the next render to process it.
    /// It's stored here in between key press and processing.
    pending_key_event: Option<KeyEvent>,

    /// Used to indicate that we need to do some state setup during render.
    first_render: bool,
}

impl State {
    pub fn new(default_project_id: &ProjectId) -> Self {
        let mut tree = TreeState::default();
        tree.select(vec![default_project_id.clone()]);

        Self {
            tree,
            pending_key_event: None,
            first_render: true,
        }
    }

    pub fn selected_id(&self) -> Option<ProjectId> {
        self.tree.selected().into_iter().last()
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.pending_key_event = Some(key);
    }

    fn handle_key_later(&mut self, key: KeyEvent, tree_items: &[TreeItem<'_, ProjectId>]) {
        match key.code {
            KeyCode::Char('\n' | ' ') => self.tree.toggle_selected(),
            KeyCode::Left => self.tree.key_left(),
            KeyCode::Right => self.tree.key_right(),
            KeyCode::Down => self.tree.key_down(tree_items),
            KeyCode::Up => self.tree.key_up(tree_items),
            _ => {}
        }
    }
}

#[derive(Debug, Default)]
pub struct Widget<'a> {
    marker: std::marker::PhantomData<(&'a mut AppState, &'a mut Model)>,
}

impl<'a> StatefulWidget for Widget<'a> {
    type State = (&'a mut AppState, &'a mut Model);

    /// Renders the app state into a terminal frame.
    ///
    /// # Panics
    /// If the model contains projects with duplicate ids
    fn render(self, area: Rect, buf: &mut Buffer, (app_state, model): &mut Self::State) {
        // recursive helper function
        fn build_tree<'b>(
            projects: &'_ [Project],
            parent_id: Option<&ProjectId>,
        ) -> Vec<TreeItem<'b, ProjectId>> {
            projects
                .iter()
                .filter_map(|project| {
                    if project.parent_id.as_ref() == parent_id {
                        // TODO : sort by `project.child_order`
                        let children = build_tree(projects, Some(&project.id));
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

        if app_state.projects.first_render {
            app_state.projects.first_render = false;

            for project in &mut model.projects {
                if !project.collapsed {
                    app_state.projects.tree.open(vec![project.id.clone()]);
                }
            }
        }

        let tree_items = build_tree(&model.projects, None);

        // now that we've made the tree items, we can handle key events, some of which require
        // the tree_items to be present.
        if let Some(key) = app_state.projects.pending_key_event.take() {
            app_state.projects.handle_key_later(key, &tree_items);
        }

        let focused = app_state.mode == Mode::SelectingProjects;

        let tree = Tree::new(tree_items)
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

        tree.render(area, buf, &mut app_state.projects.tree);
    }
}
