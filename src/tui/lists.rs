use crossterm::event::{self, KeyCode};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::model::{item::Item, project::Project};

#[derive(Default)]
pub struct State {
    pub state: ListState,
    pub length: usize,
}

impl State {
    #[must_use]
    pub fn new(length: usize, selected: usize) -> Self {
        Self {
            state: ListState::with_selected(ListState::default(), Some(selected)),
            length,
        }
    }

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
            Some(index) if self.length > 0 => {
                if index >= self.length {
                    Some(self.length - 1)
                } else {
                    Some(index)
                }
            }
            _ => None,
        };
        self.state.select(new_index);
    }

    fn select_next(&mut self) {
        if self.length == 0 {
            return;
        }

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
        if self.length == 0 {
            return;
        }

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

#[must_use]
pub fn item_list<'a>(items: &'a [&'a Item], project_name: &'a str, focused: bool) -> List<'a> {
    let list_items: Vec<ListItem> = items.iter().map(render_item).collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(project_name)
        .border_style(Style::default().fg(if focused { Color::Yellow } else { Color::Gray }));
    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .block(block);
    list
}

fn render_item<'a>(item: &'a &'a Item) -> ListItem<'a> {
    let mut spans = vec![
        Span::raw(if item.checked { "✓ " } else { "- " }),
        Span::raw(&item.content),
    ];

    if let Some(due_date) = &item.due {
        spans.push(Span::styled(
            format!("  ({due_date})"),
            Style::default().fg(Color::Gray),
        ));
    }
    let mut list_item = ListItem::new(Line::from(spans));
    if item.checked {
        list_item = list_item.style(Style::default().fg(Color::Green));
    }
    list_item
}

#[must_use]
pub fn project_list<'a>(projects: &'a [&'a Project], focused: bool) -> List {
    let mut root_projects = Vec::from(projects);
    // sort by child_id
    root_projects.sort_unstable_by_key(|project: &&Project| project.child_order);
    // top level-- only the projects without a parent
    root_projects.retain(|project| project.parent_id.is_none());

    // TODO 2022-10-21 : this needs some sort of tree structure...
    // right now it only shows first-level children
    //
    // IDEA: Create a trait called `Hierarchical` or something with two methods:
    // - `child_order()`
    // - `parent_id()`
    // (requires making project::Id a trait as well? so it can gel with the eventual item::Id)
    // anyways, then we could implement some kind of "into_tree" method on that trait that
    // takes care of both items and projects
    // ... or just copy this code to use items, whatever
    let list_items: Vec<ListItem> = root_projects
        .iter()
        .flat_map(|project| {
            let mut list = vec![ListItem::new(&project.name[..])];
            let mut children: Vec<ListItem<'_>> = projects
                .iter()
                .filter(|child| child.parent_id.as_ref().is_some_and(|id| id == &project.id))
                .map(|project| ListItem::new(format!("  {}", &project.name[..])))
                .collect();
            list.append(&mut children);
            list
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Projects")
        .border_style(Style::default().fg(if focused { Color::Yellow } else { Color::Gray }));
    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .block(block);
    list
}
