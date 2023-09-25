use crate::model::Model;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::Backend,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use super::ui::centered_rect;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AddingTodo,
    Chillin,
    Exiting,
}

/// Manages the UI state and data model
pub struct App<'a> {
    pub mode: Mode,
    model: &'a mut Model,
    input: Input,
}

impl<'a> App<'a> {
    pub fn new(model: &'a mut Model) -> Self {
        Self {
            mode: Mode::Chillin,
            model,
            input: Input::default(),
        }
    }

    /// Manages how the whole app reacts to an individual user keypress.
    /// (For now,) returns true if the app should exit.
    pub fn handle_key(&mut self, key: event::KeyEvent) {
        match self.mode {
            Mode::Chillin => match key.code {
                KeyCode::Char('a') => {
                    self.mode = Mode::AddingTodo;
                }
                KeyCode::Char('q') => {
                    self.mode = Mode::Exiting;
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
    pub fn render<'b, B: Backend + 'b>(&self, frame: &mut Frame<'b, B>) {
        let inbox_items: Vec<ListItem> = self
            .model
            .get_inbox_items()
            .iter()
            .map(|item| ListItem::new(&item.content[..]))
            .collect();

        let message = Paragraph::new("It's TUI time babyyyyy");
        let inbox_list =
            List::new(inbox_items).block(Block::default().borders(Borders::ALL).title("Inbox"));
        frame.render_widget(message, frame.size());
        frame.render_widget(inbox_list, frame.size());

        let input_rect = centered_rect(frame.size(), 50, 3, Some(2));

        if self.mode == Mode::AddingTodo {
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
