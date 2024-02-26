use super::{
    app_state::{AppState, Mode},
    item_input::ItemInput,
    ui::centered_rect,
    widgets::{items, key_hints, projects},
};
use crate::model::{project::Project, Model};
use chrono::{Local, NaiveDate};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{Constraint, Direction, Layout},
    Frame,
};

/// Manages the UI state and data model
pub struct App<'a> {
    pub model: &'a mut Model,
    pub state: AppState,
    item_input: ItemInput,
}

impl<'a> App<'a> {
    /// # Panics
    /// If the model contains projects or items with duplicate ids
    pub fn new(model: &'a mut Model) -> Self {
        let projects_state = projects::State::new(&model.inbox_project().id);

        let items_state = items::State::default();

        let state = AppState {
            projects: projects_state,
            mode: Mode::SelectingItems,
            items: items_state,
        };

        Self {
            model,
            state,
            item_input: ItemInput::new(Local::now().date_naive()),
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
        self.state
            .projects
            .selected_id()
            .and_then(|id| self.model.project_with_id(&id))
    }

    /// Updates the inner state of model after the model changes.
    pub fn update_state(&mut self) {
        let item_state = items::State::default();

        self.state.items = item_state;
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
                }
                KeyCode::Char(' ') => {
                    if let Some(item_id) = self.state.items.selected_item_id() {
                        // FIXME: toggle, don't always set to true
                        self.model.mark_item(&item_id.clone(), true);
                        self.update_state();
                    }
                }
                _ => {
                    self.state.items.handle_key(key);
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
                    let project_id = self
                        .selected_project()
                        .unwrap_or_else(|| self.model.inbox_project())
                        .id
                        .clone();

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
        frame.render_stateful_widget(
            projects::Widget::default(),
            main_left,
            &mut (&mut self.state, self.model),
        );

        // item list
        frame.render_stateful_widget(
            items::Widget::default(),
            main_right,
            &mut (&mut self.state, self.model),
        );

        // key hints
        frame.render_stateful_widget(key_hints::Widget::default(), bottom_panel, &mut self.state);

        // input bar (if adding something)
        if self.state.mode == Mode::AddingItem {
            let input_rect = centered_rect(frame.size(), 50, 3, Some(2));
            frame.render_widget(self.item_input.clone(), input_rect);
            let cursor_position = self.item_input.cursor_position(input_rect);
            frame.set_cursor(cursor_position.0, cursor_position.1);
        }
    }
}
