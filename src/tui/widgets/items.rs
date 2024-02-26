use super::section::{State as SectionState, Widget as SectionWidget};
use crate::{
    model::{
        item::{Id as ItemId, Item},
        section::{Id as SectionId, Section},
        Model,
    },
    tui::app_state::{AppState, Mode},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Position, Size},
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, StatefulWidget, Widget as RatatuiWidget},
};
use tui_scrollview::{ScrollView, ScrollViewState};
use tui_tree_widget::TreeItem;

#[derive(Debug, Clone, Default)]
pub struct State {
    /// The states of the all of the sections in the currently-selected project
    section_states: Vec<SectionState>,

    /// `None` indicates that the set of items with no section is the "section" that's currently selected
    current_section_id: Option<SectionId>,

    /// Stores where the widget is scrolled
    scroll: ScrollViewState,

    /// When a key is pressed, we wait until the next render to process it.
    /// It's stored here in between key press and processing.
    pending_key_event: Option<KeyEvent>,
}

impl State {
    pub fn selected_item_id(&self) -> Option<ItemId> {
        let x = self
            .current_section_state()
            .and_then(SectionState::selected_item_id);
        assert!(x.is_some());
        x
    }

    fn current_section_state(&self) -> Option<&SectionState> {
        self.section_states
            .iter()
            .find(|section_state| section_state.id == self.current_section_id)
    }

    fn recompute_scroll_offset(
        &mut self,
        section_states_and_tree_items: &[(SectionState, Vec<TreeItem<'_, ItemId>>)],
    ) {
        let current_tree_items = section_states_and_tree_items
            .iter()
            .find_map(|(s, t)| {
                if s.id == self.current_section_id {
                    Some(t)
                } else {
                    None
                }
            })
            .expect("currently-selected section id should always match an existing section");

        #[allow(clippy::cast_possible_truncation)]
        let offset = self
            .current_section_state()
            .map_or(0, |s| s.offset(current_tree_items) as u16);

        let mut offset = section_states_and_tree_items
            .iter()
            .take_while(|(section, _)| section.id != self.current_section_id)
            .map(|(section_state, tree_items)| section_state.height(tree_items))
            .sum::<u16>()
            + offset;

        // HACK: (maybe?) don't scroll the view until the selection has moved down some
        offset = offset.saturating_sub(5);

        self.scroll.set_offset(Position::new(0, offset));
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.pending_key_event = Some(key);
    }

    pub fn handle_key_later(
        &mut self,
        key: KeyEvent,
        sections_and_items: &[(Option<&Section>, Vec<&Item>)],
    ) {
        let section_states_and_tree_items: Vec<(SectionState, Vec<TreeItem<'_, ItemId>>)> =
            sections_and_items
                .iter()
                .map(|(section, items)| {
                    let section_state = self
                        .section_states
                        .iter()
                        .find(|state| match (&state.id, section) {
                            (Some(id), Some(section)) => section.id == *id,
                            (None, None) => true,
                            _ => false,
                        })
                        .expect("we just did this!")
                        .clone();
                    let tree_items = SectionState::build_tree(items, None);
                    (section_state, tree_items)
                })
                .collect();

        let (current_index, current_section_state) = self
            .section_states
            .iter_mut()
            .enumerate()
            .find(|(_, section_state)| section_state.id == self.current_section_id)
            .expect("currently-selected section id should always match an existing section");

        let current_tree_items = section_states_and_tree_items
            .iter()
            .find_map(|(s, t)| {
                if s.id == self.current_section_id {
                    Some(t)
                } else {
                    None
                }
            })
            .expect("currently-selected section id should always match an existing section");

        let at_boundary = current_section_state.handle_key(key, current_tree_items);
        if at_boundary {
            let next_index = match key.code {
                KeyCode::Down => current_index
                    .saturating_add(1)
                    .min(self.section_states.len() - 1),
                KeyCode::Up => current_index.saturating_sub(1).max(0),
                _ => 0,
            };

            self.current_section_id = self
                .section_states
                .get(next_index)
                .map(|x| x.id.clone())
                .unwrap_or_default();
        }

        self.recompute_scroll_offset(&section_states_and_tree_items);
    }
}

#[derive(Debug, Default)]
pub struct Widget<'a> {
    marker: std::marker::PhantomData<(&'a mut AppState, &'a mut Model)>,
}

impl<'a> StatefulWidget for Widget<'a> {
    type State = (&'a mut AppState, &'a mut Model);

    fn render(self, area: Rect, buf: &mut Buffer, (app_state, model): &mut Self::State) {
        // if no project is selected, just bail
        // TODO: render the border first, then bail lol
        let Some(project_id) = app_state.projects.selected_id() else {
            return;
        };

        // get a list of sections in the selected project paired with the items in that section
        let sections_and_items = model.sections_and_items_in_project(&project_id);

        // find the section states for each section, or (as is the case on the first render)
        // create a new one. then compute the tree of items in each section
        for (section, items) in &sections_and_items {
            if !app_state
                .items
                .section_states
                .iter()
                .any(|state| match (&state.id, section) {
                    (Some(id), Some(section)) => section.id == *id,
                    (None, None) => true,
                    _ => false,
                })
            {
                app_state
                    .items
                    .section_states
                    .push(SectionState::new(*section, items));
            }
        }

        // we can now handle a key event if there is one
        if let Some(key) = app_state.items.pending_key_event.take() {
            app_state.items.handle_key_later(key, &sections_and_items);
        }

        let mut section_states_and_tree_items: Vec<(&SectionState, Vec<TreeItem<'_, ItemId>>)> =
            sections_and_items
                .iter()
                .map(|(section, items)| {
                    let section_state = app_state
                        .items
                        .section_states
                        .iter()
                        .find(|state| match (&state.id, section) {
                            (Some(id), Some(section)) => section.id == *id,
                            (None, None) => true,
                            _ => false,
                        })
                        .expect("we just did this!");
                    let tree_items = SectionState::build_tree(items, None);
                    (section_state, tree_items)
                })
                .collect();

        let heights = section_states_and_tree_items
            .iter()
            .map(|(section_state, tree_items)| section_state.height(tree_items));
        let total_height = heights.clone().sum::<u16>();
        let constraints = Constraint::from_mins(heights);

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Tasks")
            .border_style(
                Style::default().fg(if app_state.mode == Mode::SelectingItems {
                    Color::Yellow
                } else {
                    Color::Gray
                }),
            );

        // render the border
        let area = {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        };

        // render the rest of this widget into the buffer of a scroll view
        let mut scrollview = ScrollView::new(Size::new(area.width, total_height));

        // split into pieces based on section heights
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(scrollview.area());

        // render inside each piece
        section_states_and_tree_items
            .iter_mut()
            .enumerate()
            .for_each(|(index, (section_state, tree_items))| {
                if let Some(rect) = layout.get(index) {
                    SectionWidget::new(app_state.items.current_section_id.clone(), tree_items)
                        .render(*rect, scrollview.buf_mut(), &mut (section_state, model));
                }
            });

        // render the whole deal into the scrollview
        scrollview.render(area, buf, &mut app_state.items.scroll);
    }
}
