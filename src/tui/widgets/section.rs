use crate::model::{
    item::{Id as ItemId, Item},
    section::{Id as SectionId, Section},
    Model,
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
pub struct State {
    pub id: Option<SectionId>,
    name: Option<String>,
    tree: TreeState<ItemId>,
}

impl State {
    pub fn new(section: Option<&Section>, items: &'_ [&Item]) -> Self {
        let tree_items = Self::build_tree(items, None);

        let mut tree_state = TreeState::default();

        for item in items {
            if !item.collapsed {
                tree_state.open(vec![item.id.clone()]);
            }
        }

        tree_state.select_first(&tree_items);

        Self {
            id: section.map(|s| s.id.clone()),
            name: section.map(|s| s.name.clone()),
            tree: tree_state,
        }
    }

    pub fn build_tree<'b>(
        items: &'_ [&Item],
        parent_id: Option<&ItemId>,
    ) -> Vec<TreeItem<'b, ItemId>> {
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
    pub fn height(&self, tree_items: &[TreeItem<'_, ItemId>]) -> u16 {
        // add one line for the section title if it's there and one for a spacer after the section
        (self
            .tree
            .flatten(tree_items)
            .iter()
            .map(|f| f.item.height() as u16)
            .sum::<u16>())
            + if self.name.is_some() { 2 } else { 1 }
    }

    pub fn offset(&self, tree_items: &[TreeItem<'_, ItemId>]) -> usize {
        self.tree
            .flatten(tree_items)
            .iter()
            .position(|item| item.identifier.last() == self.selected_item_id().as_ref())
            .unwrap_or(0)
    }

    pub fn selected_item_id(&self) -> Option<ItemId> {
        self.tree.selected().last().cloned()
    }

    pub fn handle_key(&mut self, key: KeyEvent, tree_items: &[TreeItem<'_, ItemId>]) -> bool {
        match key.code {
            KeyCode::Char('\n' | ' ') => {
                self.tree.toggle_selected();
            }
            KeyCode::Left => self.tree.key_left(),
            KeyCode::Right => self.tree.key_right(),
            KeyCode::Down => {
                let before = self.tree.selected();
                self.tree.key_down(tree_items);
                let after = self.tree.selected();

                if before == after {
                    return true;
                }
            }
            KeyCode::Up => {
                let before = self.tree.selected();
                self.tree.key_up(tree_items);
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
pub struct Widget<'a, 'b> {
    /// The id of the section currently focused by the app
    focused_id: Option<SectionId>,

    /// The items in this section, as
    tree_items: &'b [TreeItem<'a, ItemId>],

    marker: std::marker::PhantomData<(&'b mut State, &'b mut Model)>,
}

impl<'a, 'b> Widget<'a, 'b> {
    pub fn new(focused_id: Option<SectionId>, tree_items: &'b [TreeItem<'a, ItemId>]) -> Self {
        Self {
            focused_id,
            tree_items,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a, 'b: 'a> StatefulWidget for Widget<'a, 'b> {
    type State = (&'b State, &'b mut Model);

    fn render(self, area: Rect, buf: &mut Buffer, (state, _): &mut Self::State) {
        let mut tree = Tree::new(self.tree_items.to_vec()).expect("Item ids must be unique");

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

        StatefulWidget::render(tree, tree_area, buf, &mut state.tree.clone());
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
