use crate::{
    model::{
        item::{Id as ItemId, Item},
        section::{Id as SectionId, Section},
    },
    tui::app_state::AppState,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, StatefulWidget, Widget as RatatuiWidget},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

// pub type State = TreeState<ItemId>;

#[derive(Debug, Clone)]
pub struct State<'a> {
    pub id: Option<SectionId>,
    name: Option<String>,
    tree: TreeState<ItemId>,
    items: Vec<TreeItem<'a, ItemId>>,
}

impl<'a> State<'a> {
    pub fn new(section: Option<&Section>, items: &'_ [&Item]) -> Self {
        let tree_items = Self::build_tree(items, None);

        let mut state = TreeState::default();

        for item in items {
            if !item.collapsed {
                state.open(vec![item.id.clone()]);
            }
        }

        state.select_first(&tree_items);

        Self {
            id: section.map(|s| s.id.clone()),
            name: section.map(|s| s.name.clone()),
            items: tree_items,
            tree: state,
        }
    }

    fn build_tree<'b>(items: &'_ [&Item], parent_id: Option<&ItemId>) -> Vec<TreeItem<'b, ItemId>> {
        items
            .iter()
            .filter_map(|item| {
                if item.parent_id.as_ref() == parent_id {
                    // TODO : sort by `item.child_order`? or should that be done in the model?
                    let children = Self::build_tree(items, Some(&item.id));
                    Some(
                        // TODO: format the item's text
                        TreeItem::new(item.id.clone(), Into::<Text>::into(*item), children)
                            .expect("Item ids must be unique"),
                    )
                } else {
                    None
                }
            })
            .collect()
    }

    /// Computes the height (in lines) of this widget
    #[allow(clippy::cast_possible_truncation)]
    pub fn height(&'a self) -> u16 {
        // add one line for the section title if it's there and one for a spacer after the section
        (self
            .tree
            .flatten(&self.items)
            .iter()
            .map(|f| f.item.height() as u16)
            .sum::<u16>())
            + if self.name.is_some() { 2 } else { 1 }
    }

    pub fn selected_item_id(&self) -> Option<ItemId> {
        self.tree.selected().last().cloned()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('\n' | ' ') => {
                self.tree.toggle_selected();
            }
            KeyCode::Left => self.tree.key_left(),
            KeyCode::Right => self.tree.key_right(),
            KeyCode::Down => {
                let before = self.tree.selected();
                self.tree.key_down(&self.items);
                let after = self.tree.selected();

                if before == after {
                    return true;
                }
            }
            KeyCode::Up => {
                let before = self.tree.selected();
                self.tree.key_up(&self.items);
                let after = self.tree.selected();

                if before == after {
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

#[derive(Debug, Default)]
pub struct Widget<'a> {
    /// The id of the section currently focused by the app
    focused_id: Option<SectionId>,

    marker: std::marker::PhantomData<AppState<'a>>,
}

impl<'a> Widget<'a> {
    pub fn new(id: Option<SectionId>) -> Self {
        Self {
            focused_id: id,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a> StatefulWidget for Widget<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut tree = Tree::new(state.items.clone()).expect("Item ids must be unique");

        // only highlight things in the currently-focused section
        if state.id == self.focused_id {
            tree = tree.highlight_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );
        }

        let tree_area = if let Some(ref name) = state.name {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(1), Constraint::Min(1)])
                .split(area);

            let title_area = layout[0];
            let tree_area = layout[1];

            let title = Paragraph::new(name.clone());
            title.render(title_area, buf);

            tree_area
        } else {
            area
        };

        StatefulWidget::render(tree, tree_area, buf, &mut state.tree);
    }
}

impl From<&Item> for Text<'_> {
    fn from(item: &Item) -> Self {
        let mut spans = vec![
            Span::raw(if item.checked { "✓ " } else { "- " }),
            Span::raw(item.content.clone()),
        ];

        if let Some(due_date) = &item.due {
            spans.push(Span::styled(
                format!("  ({due_date})"),
                Style::default().fg(Color::Gray),
            ));
        }
        let mut text = Text::from(Line::from(spans));
        if item.checked {
            text = text.style(Style::default().fg(Color::Green));
        }
        text
    }
}
