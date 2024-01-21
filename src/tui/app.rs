use super::{
    item_input::ItemInput,
    key_hints::KeyHint,
    lists::{item_list, State as ListState},
    projects_pane::ProjectTree,
    ui::centered_rect,
};
use crate::model::{item::Item, project::Project, Model};
use chrono::{Local, NaiveDate};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AddingItem,
    SelectingItems,
    SelectingProjects,
    Exiting,
}

/// Manages the UI state and data model
pub struct App<'a> {
    pub model: &'a mut Model,
    pub mode: Mode,
    item_input: ItemInput,
    item_list_state: ListState,
    project_tree: ProjectTree<'a>,
}

impl<'a> App<'a> {
    /// # Panics
    /// If the model contains projects or items with duplicate ids
    pub fn new(model: &'a mut Model) -> Self {
        let num_items = model.get_inbox_items(false).len();
        let project_tree = ProjectTree::new(&model.projects);

        Self {
            mode: Mode::SelectingItems,
            model,
            item_input: ItemInput::new(Local::now().date_naive()),
            item_list_state: ListState::with_length(num_items),
            project_tree,
        }
    }

    /// FIXME: This is used for testing; is there a more elegant way to accomplish that?
    pub fn new_with_date(model: &'a mut Model, today: NaiveDate) -> Self {
        Self {
            item_input: ItemInput::new(today),
            ..Self::new(model)
        }
    }

    fn selected_project(&self) -> Option<&Project> {
        self.project_tree.selected().and_then(|project_id| {
            self.model
                .projects
                .iter()
                .find(|project| project.id == project_id)
        })
    }

    /// Updates the inner state of model after the model changes.
    pub fn update_state(&mut self) {
        let num_items = self.items_in_selected_project().len();
        self.item_list_state.set_length(num_items);
    }

    fn items_in_selected_project(&self) -> Vec<&Item> {
        self.selected_project()
            .map(|project| self.model.get_items_in_project(&project.id))
            .unwrap_or_default()
    }

    /// Manages how the whole app reacts to an individual user keypress.
    pub fn handle_key(&mut self, key: event::KeyEvent) {
        match self.mode {
            Mode::SelectingItems => match key.code {
                KeyCode::Char('a') => {
                    self.mode = Mode::AddingItem;
                }
                KeyCode::Char('q') => {
                    self.mode = Mode::Exiting;
                }
                KeyCode::Up | KeyCode::Down => self.item_list_state.handle_key(key),
                KeyCode::Tab | KeyCode::Left => {
                    self.mode = Mode::SelectingProjects;
                }
                KeyCode::Char(' ') => {
                    if let Some(selected_index) = self.item_list_state.selected_index() {
                        let items = self.items_in_selected_project();
                        let item = items[selected_index];
                        self.model.mark_item(&item.id.clone(), !item.checked);
                        self.update_state();
                    }
                }
                _ => {}
            },
            Mode::SelectingProjects => {
                match key.code {
                    KeyCode::Char('a') => {
                        self.mode = Mode::AddingItem;
                    }
                    KeyCode::Char('q') => {
                        self.mode = Mode::Exiting;
                    }
                    KeyCode::Tab => {
                        self.mode = Mode::SelectingItems;
                    }
                    _ => {
                        self.project_tree.handle_key(key);
                    }
                }
                self.update_state();
            }
            Mode::AddingItem => match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::SelectingItems;
                    self.item_input.reset();
                }
                KeyCode::Enter => {
                    let selected_project = self.selected_project();
                    if let Some(selected_project) = selected_project {
                        let project_id = selected_project.id.clone();

                        let (content, due_date) = self.item_input.get_new_item();

                        self.model.add_item(content.trim(), project_id, due_date);
                        self.update_state();
                        self.mode = Mode::SelectingItems;
                        self.item_input.reset();
                    }
                }
                _ => {
                    self.item_input.handle_event(&Event::Key(key));
                }
            },
            Mode::Exiting => {}
        }
    }

    /// Renders the app state into a terminal frame.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Max(1)])
            .split(frame.size());
        let main_panel = layout[0];
        let bottom_panel = layout[1];

        let main_panel_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_panel);
        let main_left = main_panel_layout[0];
        let main_right = main_panel_layout[1];

        // project list
        let project_tree = self.project_tree.tree(self.mode == Mode::SelectingProjects);
        let project_state = self.project_tree.state_mut();
        frame.render_stateful_widget(project_tree, main_left, project_state);

        // item list
        let items = self.items_in_selected_project();
        let item_list = item_list(
            &items,
            self.selected_project()
                .map_or("", |project| &project.name[..]),
            &self.model.sections,
            self.mode == Mode::SelectingItems,
        );
        // TODO: make trees for items, then resolve this borrow issue
        // frame.render_stateful_widget(item_list, main_right, &mut self.item_list_state.state);
        frame.render_widget(item_list, main_right);

        // key hints
        let key_hints = KeyHint::from_mode(&self.mode);
        let key_hint_line: Line = Line::from(
            key_hints
                .into_iter()
                .flat_map(Into::<Vec<Span>>::into)
                .collect::<Vec<Span>>(),
        );
        frame.render_widget(Paragraph::new(key_hint_line), bottom_panel);

        // input bar (if adding something)
        if self.mode == Mode::AddingItem {
            let input_rect = centered_rect(frame.size(), 50, 3, Some(2));
            frame.render_widget(self.item_input.clone(), input_rect);
            let cursor_position = self.item_input.cursor_position(input_rect);
            frame.set_cursor(cursor_position.0, cursor_position.1);
        }
    }
}
