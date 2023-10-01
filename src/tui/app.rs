use super::{
    item_list::{ItemList, State as ItemListState},
    ui::centered_rect,
};
use crate::model::Model;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AddingTodo,
    Chillin,
    Exiting,
}

/// Manages the UI state and data model
pub struct App<'a> {
    pub mode: Mode,
    pub model: &'a mut Model,
    input: Input,
    item_list_state: ItemListState,
}

impl<'a> App<'a> {
    pub fn new(model: &'a mut Model) -> Self {
        let length = model.get_inbox_items(false).len();
        Self {
            mode: Mode::Chillin,
            model,
            input: Input::default(),
            item_list_state: ItemListState::with_length(length),
        }
    }

    /// Updates the inner state of model after the model changes.
    pub fn update_state(&mut self) {
        self.item_list_state
            .set_length(self.model.get_inbox_items(false).len());
    }

    /// Manages how the whole app reacts to an individual user keypress.
    pub fn handle_key(&mut self, key: event::KeyEvent) {
        match self.mode {
            Mode::Chillin => match key.code {
                KeyCode::Char('a') => {
                    self.mode = Mode::AddingTodo;
                }
                KeyCode::Char('q') => {
                    self.mode = Mode::Exiting;
                }
                KeyCode::Up | KeyCode::Down => self.item_list_state.handle_key(key),
                KeyCode::Char(' ') => {
                    if let Some(selected_index) = self.item_list_state.selected_index() {
                        let item = self.model.get_inbox_items(false)[selected_index];
                        self.model.mark_item(&item.id.clone(), !item.checked);
                        self.update_state();
                    }
                }
                _ => {}
            },
            Mode::AddingTodo => match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::Chillin;
                    self.input.reset();
                }
                KeyCode::Enter => {
                    self.model.add_item(self.input.value());
                    self.update_state();
                    self.mode = Mode::Chillin;
                    self.input.reset();
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

        let project_list_items: Vec<ListItem> = self
            .model
            .projects
            .iter()
            .map(|project| ListItem::new(&project.name[..]))
            .collect();
        let project_list = List::new(project_list_items)
            .block(Block::default().borders(Borders::ALL).title("Projects"));
        frame.render_widget(project_list, chunks[0]);

        let mut inbox_component = ItemList {
            items: self.model.get_inbox_items(false),
            state: &mut self.item_list_state,
        };
        inbox_component.render(frame, chunks[1]);

        if self.mode == Mode::AddingTodo {
            let input_rect = centered_rect(frame.size(), 50, 3, Some(2));
            let input_scroll = self.input.visual_scroll(input_rect.width as usize - 2);
            #[allow(clippy::cast_possible_truncation)]
            let input = Paragraph::new(self.input.value())
                .scroll((0, input_scroll as u16))
                .block(Block::default().title("New Todo").borders(Borders::ALL));
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