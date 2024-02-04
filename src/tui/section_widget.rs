use crate::model::item::{Id as ItemId, Item};
use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Paragraph, StatefulWidget, Widget},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

pub type State = TreeState<ItemId>;

#[derive(Debug, Clone)]
pub struct SectionWidget<'a> {
    /// Name of the section, or `None` if it's the top-level section
    name: Option<String>,

    /// A tree of the items in the section
    items: Vec<TreeItem<'a, ItemId>>,
}

impl<'a> SectionWidget<'a> {
    pub fn new(name: Option<String>, items: &'_ [&Item]) -> Self {
        let tree_items = Self::build_tree(items, None);

        Self {
            name,
            items: tree_items,
        }
    }

    fn build_tree<'b>(items: &'_ [&Item], parent_id: Option<&ItemId>) -> Vec<TreeItem<'b, ItemId>> {
        items
            .iter()
            .filter_map(|item| {
                if item.parent_id.as_ref() == parent_id {
                    // TODO : sort by `project.child_order`
                    let children = Self::build_tree(items, Some(&item.id));
                    Some(
                        TreeItem::new(item.id.clone(), item.content.clone(), children)
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
    pub fn height(&'a self, state: &State) -> u16 {
        // add one line for the section title if it's there and one for a spacer after the section
        (state
            .flatten(&self.items)
            .iter()
            .map(|f| f.item.height() as u16)
            .sum::<u16>())
            + if self.name.is_some() { 2 } else { 1 }
    }
}

impl<'a> StatefulWidget for SectionWidget<'a> {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let tree = Tree::new(self.items.clone())
            .expect("Item ids must be unique")
            .highlight_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );

        let tree_area = if let Some(name) = self.name {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(1), Constraint::Min(1)])
                .split(area);

            let title_area = layout[0];
            let tree_area = layout[1];

            let title = Paragraph::new(name);
            title.render(title_area, buf);

            tree_area
        } else {
            area
        };

        StatefulWidget::render(tree, tree_area, buf, state);
    }
}
