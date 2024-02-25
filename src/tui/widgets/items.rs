use super::section::{State as SectionState, Widget as SectionWidget};
use crate::{
    model::{
        item::{Id as ItemId, Item},
        project::Project,
        section::{Id as SectionId, Section},
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

#[derive(Debug, Clone)]
pub struct State<'a> {
    pub section_states: Vec<SectionState<'a>>,

    /// `None` indicates that the set of items with no section is the "section" that's currently selected
    pub current_section_id: Option<SectionId>,

    scroll: ScrollViewState,
}

impl<'a> State<'a> {
    pub fn new(items: &'_ [Item], sections: &'_ [Section], project: &Project) -> Self {
        let mut section_state_pairs: Vec<_> = sections
            .iter()
            .filter(|section| section.project_id == project.id)
            .map(|section| {
                let items_in_section: Vec<_> = items
                    .iter()
                    .filter(|item| item.section_id.as_ref().is_some_and(|id| id == &section.id))
                    .collect();
                let section_state = SectionState::new(Some(section), &items_in_section);
                (Some(section), section_state)
            })
            .collect();

        let items_in_no_section: Vec<_> = items
            .iter()
            .filter(|item| item.project_id == project.id)
            .filter(|item| item.section_id.is_none())
            .collect();
        let section_state = SectionState::new(None, &items_in_no_section);
        section_state_pairs.insert(0, (None, section_state));

        section_state_pairs.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

        let section_states = section_state_pairs
            .into_iter()
            .map(|(_, state)| state)
            .collect();

        Self {
            current_section_id: None,
            section_states,
            scroll: ScrollViewState::default(),
        }
    }

    pub fn selected_item_id(&self) -> Option<ItemId> {
        self.current_section_state()
            .and_then(SectionState::selected_item_id)
    }

    fn current_section_state(&self) -> Option<&SectionState<'_>> {
        self.section_states
            .iter()
            .find(|section_state| section_state.id == self.current_section_id)
    }

    fn recompute_scroll_offset(&mut self) {
        #[allow(clippy::cast_possible_truncation)]
        let offset = self
            .current_section_state()
            .map_or(0, |s| s.offset() as u16);

        let mut offset = self
            .section_states
            .iter()
            .take_while(|section| section.id != self.current_section_id)
            .map(SectionState::height)
            .sum::<u16>()
            + offset;

        // HACK: (maybe?) don't scroll the view until the selection has moved down some
        offset = offset.saturating_sub(5);

        self.scroll.set_offset(Position::new(0, offset));
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let (current_index, current_section_state) = self
            .section_states
            .iter_mut()
            .enumerate()
            .find(|(_, section_state)| section_state.id == self.current_section_id)
            .expect("currently-selected section id should always match an existing section");

        let at_boundary = current_section_state.handle_key(key);
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

        self.recompute_scroll_offset();
    }
}

#[derive(Debug, Default)]
pub struct Widget<'a> {
    marker: std::marker::PhantomData<AppState<'a>>,
}

impl<'a> StatefulWidget for Widget<'a> {
    type State = AppState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let heights = state.items.section_states.iter().map(SectionState::height);
        let total_height = heights.clone().sum::<u16>();
        let constraints = Constraint::from_mins(heights);

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Tasks")
            .border_style(Style::default().fg(if state.mode == Mode::SelectingItems {
                Color::Yellow
            } else {
                Color::Gray
            }));

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
        state
            .items
            .section_states
            .iter_mut()
            .enumerate()
            .for_each(|(index, section_state)| {
                // FIXME (maybe) : this just doesn't render stuff if it can't find the layout or
                // state -- is that good? (better than crashing I guess)
                if let Some(rect) = layout.get(index) {
                    SectionWidget::new(state.items.current_section_id.clone()).render(
                        *rect,
                        scrollview.buf_mut(),
                        section_state,
                    );
                }
            });

        // render the whole deal into the scrollview
        scrollview.render(area, buf, &mut state.items.scroll);
    }
}
