use crossterm::event::{self, KeyCode};
use ratatui::{
    prelude::Backend,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::model::item::Item;

#[derive(Default)]
pub struct ItemListState {
    state: ListState,
    length: usize,
}

impl ItemListState {
    #[must_use]
    pub fn with_length(length: usize) -> Self {
        Self {
            state: ListState::default(),
            length,
        }
    }

    pub fn set_length(&mut self, length: usize) {
        self.length = length;
        let new_index = match self.state.selected() {
            Some(index) => {
                if index >= self.length {
                    Some(self.length - 1)
                } else {
                    Some(index)
                }
            }
            None => None,
        };
        self.state.select(new_index);
    }

    fn select_next(&mut self) {
        let next_index = match self.state.selected() {
            Some(index) => {
                if index >= self.length - 1 {
                    0
                } else {
                    index + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(next_index));
    }

    fn select_previous(&mut self) {
        let previous_index = match self.state.selected() {
            Some(index) => {
                if index == 0 {
                    self.length - 1
                } else {
                    index - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(previous_index));
    }

    pub fn handle_key(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),
            _ => {}
        }
    }

    #[must_use]
    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }
}

pub struct ItemList<'a> {
    pub items: Vec<&'a Item>,
    pub state: &'a mut ItemListState,
}

impl<'a> ItemList<'a> {
    // TODO: alternative to this function: implement `Widget` for ItemList
    pub fn render<'b, B: Backend + 'b>(&mut self, frame: &mut Frame<'b, B>) {
        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                let check = if item.checked { "✓" } else { "-" };
                let mut list_item = ListItem::new(format!("{check} {}", &item.content[..]));
                if item.checked {
                    list_item = list_item.style(Style::default().fg(Color::Green));
                }
                list_item
            })
            .collect();
        let list = List::new(list_items)
            .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
            .block(Block::default().borders(Borders::ALL).title("Inbox"));

        frame.render_stateful_widget(list, frame.size(), &mut self.state.state);
    }
}
