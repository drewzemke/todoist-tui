use super::{
    app_state::{AppState, Mode},
    item_input::ItemInput,
    items_pane::{ItemTree, ItemTreeState},
    key_hints::KeyHint,
    projects_pane::{ProjectsPane, ProjectsState},
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

/// Manages the UI state and data model
pub struct App<'a> {
    pub model: &'a mut Model,
    pub state: AppState<'a>,
    item_input: ItemInput,
    item_tree: ItemTree<'a>,
    item_tree_state: ItemTreeState,
}

impl<'a> App<'a> {
    /// # Panics
    /// If the model contains projects or items with duplicate ids
    pub fn new(model: &'a mut Model) -> Self {
        let project_state = ProjectsState::new(&model.projects);
        let selected_project_id = project_state.selected();
        let selected_project = model
            .project_with_id(&selected_project_id)
            .expect("Could not find project with id {selected_project_id}");

        let item_tree = ItemTree::new(&model.items, &model.sections, selected_project);
        let mut item_tree_state =
            ItemTreeState::new(&model.items, &model.sections, selected_project);
        item_tree_state.set_focused(true);

        let state = AppState {
            projects: ProjectsState::new(&model.projects),
            mode: Mode::SelectingItems,
        };

        Self {
            model,
            item_input: ItemInput::new(Local::now().date_naive()),
            item_tree,
            item_tree_state,
            state,
        }
    }

    /// FIXME: This is used for testing; is there a more elegant way to accomplish that?
    pub fn new_with_date(model: &'a mut Model, today: NaiveDate) -> Self {
        Self {
            item_input: ItemInput::new(today),
            ..Self::new(model)
        }
    }

    fn selected_project(&self) -> &Project {
        let selected_project_id = self.state.projects.selected();
        self.model
            .project_with_id(&selected_project_id)
            .expect("Could not find project with id {selected_project_id}")
    }

    fn selected_item(&self) -> Option<&Item> {
        self.item_tree_state
            .selected()
            .and_then(|item_id| self.model.items.iter().find(|item| item.id == item_id))
    }

    /// Updates the inner state of model after the model changes.
    pub fn update_state(&mut self) {
        let selected_project = &self.selected_project();
        let item_tree = ItemTree::new(&self.model.items, &self.model.sections, selected_project);
        let mut item_tree_state =
            ItemTreeState::new(&self.model.items, &self.model.sections, selected_project);
        item_tree_state.set_focused(self.state.mode == Mode::SelectingItems);
        self.item_tree = item_tree;
        self.item_tree_state = item_tree_state;
    }

    /// Manages how the whole app reacts to an individual user keypress.
    // TODO: move into `app_state` module
    pub fn handle_key(&mut self, key: event::KeyEvent) {
        match self.state.mode {
            Mode::SelectingItems => match key.code {
                KeyCode::Char('a') => {
                    self.state.mode = Mode::AddingItem;
                }
                KeyCode::Char('q') => {
                    self.state.mode = Mode::Exiting;
                }
                KeyCode::Tab => {
                    self.state.mode = Mode::SelectingProjects;
                    self.item_tree_state.set_focused(false);
                }
                KeyCode::Char(' ') => {
                    if let Some(item) = self.selected_item() {
                        self.model.mark_item(&item.id.clone(), !item.checked);
                        self.update_state();
                    }
                }
                _ => {
                    self.item_tree_state.handle_key(key);
                }
            },
            Mode::SelectingProjects => {
                match key.code {
                    KeyCode::Char('a') => {
                        self.state.mode = Mode::AddingItem;
                    }
                    KeyCode::Char('q') => {
                        self.state.mode = Mode::Exiting;
                    }
                    KeyCode::Tab => {
                        self.state.mode = Mode::SelectingItems;
                        self.item_tree_state.set_focused(true);
                    }
                    _ => {
                        self.state.projects.handle_key(key);
                    }
                }
                self.update_state();
            }
            Mode::AddingItem => match key.code {
                KeyCode::Esc => {
                    self.state.mode = Mode::SelectingItems;
                    self.item_input.reset();
                }
                KeyCode::Enter => {
                    let project_id = self.selected_project().id.clone();

                    let (content, due_date) = self.item_input.get_new_item();

                    self.model.add_item(content.trim(), project_id, due_date);
                    self.update_state();
                    self.state.mode = Mode::SelectingItems;
                    self.item_input.reset();
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

        // projects pane
        frame.render_stateful_widget(ProjectsPane::default(), main_left, &mut self.state);

        // item list
        frame.render_stateful_widget(
            self.item_tree.clone(),
            main_right,
            &mut self.item_tree_state,
        );

        // key hints
        let key_hints = KeyHint::from_mode(&self.state.mode);
        let key_hint_line: Line = Line::from(
            key_hints
                .into_iter()
                .flat_map(Into::<Vec<Span>>::into)
                .collect::<Vec<Span>>(),
        );
        frame.render_widget(Paragraph::new(key_hint_line), bottom_panel);

        // input bar (if adding something)
        if self.state.mode == Mode::AddingItem {
            let input_rect = centered_rect(frame.size(), 50, 3, Some(2));
            frame.render_widget(self.item_input.clone(), input_rect);
            let cursor_position = self.item_input.cursor_position(input_rect);
            frame.set_cursor(cursor_position.0, cursor_position.1);
        }
    }
}
