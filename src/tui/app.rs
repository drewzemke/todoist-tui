use super::{
    lists::{item_list, project_list, State as ListState},
    ui::centered_rect,
};
use crate::model::{item::Item, project::Project, Model};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AddingItem,
    SelectingItems,
    SelectingProjects,
    Exiting,
}

/// Manages the UI state and data model
pub struct App<'a> {
    pub mode: Mode,
    pub model: &'a mut Model,
    input: Input,
    item_list_state: ListState,
    project_list_state: ListState,
}

impl<'a> App<'a> {
    pub fn new(model: &'a mut Model) -> Self {
        let num_items = model.get_inbox_items(false).len();
        let num_projects = model.projects.len();
        Self {
            mode: Mode::SelectingItems,
            model,
            input: Input::default(),
            item_list_state: ListState::with_length(num_items),
            project_list_state: ListState::new(num_projects, 0),
        }
    }

    /// Updates the inner state of model after the model changes.
    pub fn update_state(&mut self) {
        let num_items = self.items_in_selected_project().len();
        self.item_list_state = ListState::with_length(num_items);
    }

    fn selected_project(&self) -> Option<&Project> {
        let projects = self.model.projects();
        self.project_list_state
            .selected_index()
            .map(|index| projects[index])
    }

    fn items_in_selected_project(&self) -> Vec<&Item> {
        let projects = self.model.projects();
        self.project_list_state
            .selected_index()
            .map(|index| projects[index])
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
                KeyCode::Tab => {
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
            Mode::SelectingProjects => match key.code {
                KeyCode::Char('a') => {
                    self.mode = Mode::AddingItem;
                }
                KeyCode::Char('q') => {
                    self.mode = Mode::Exiting;
                }
                KeyCode::Up | KeyCode::Down => {
                    self.project_list_state.handle_key(key);
                    self.update_state();
                }
                KeyCode::Tab => {
                    self.mode = Mode::SelectingItems;
                }
                _ => {}
            },
            Mode::AddingItem => match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::SelectingItems;
                    self.input.reset();
                }
                KeyCode::Enter => {
                    let selected_project = self.selected_project();
                    if let Some(selected_project) = selected_project {
                        let project_id = selected_project.id.clone();
                        self.model.add_item(self.input.value(), project_id);
                        self.update_state();
                        self.mode = Mode::SelectingItems;
                        self.input.reset();
                    }
                }
                _ => {
                    self.input.handle_event(&Event::Key(key));
                }
            },
            Mode::Exiting => {}
        }
    }

    /// Renders the app state into a terminal frame.
    ///
    /// # Errors
    /// Returns an error if something goes wrong during the render process.
    pub fn render<'b, B: Backend + 'b>(&mut self, frame: &mut Frame<'b, B>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.size());

        // render the project list
        let projects = self.model.projects();
        let project_list = project_list(&projects, self.mode == Mode::SelectingProjects);
        frame.render_stateful_widget(project_list, chunks[0], &mut self.project_list_state.state);

        // render the item list
        let selected_project = self
            .project_list_state
            .selected_index()
            // TODO: use `get` here
            .map(|index| projects[index]);
        if let Some(selected_project) = selected_project {
            let items = self.model.get_items_in_project(&selected_project.id);
            let item_list = item_list(
                &items,
                &selected_project.name,
                self.mode == Mode::SelectingItems,
            );
            frame.render_stateful_widget(item_list, chunks[1], &mut self.item_list_state.state);
        }

        if self.mode == Mode::AddingItem {
            let input_rect = centered_rect(frame.size(), 50, 3, Some(2));
            let input_scroll = self.input.visual_scroll(input_rect.width as usize - 2);
            #[allow(clippy::cast_possible_truncation)]
            let input = Paragraph::new(self.input.value())
                .scroll((0, input_scroll as u16))
                .block(Block::default().title("New Todo").borders(Borders::ALL));
            frame.render_widget(Clear, input_rect);
            frame.render_widget(input, input_rect);

            #[allow(clippy::cast_possible_truncation)]
            frame.set_cursor(
                input_rect.x
                    + (self.input.visual_cursor().max(input_scroll) - input_scroll) as u16
                    + 1,
                input_rect.y + 1,
            );
        }
    }
}
