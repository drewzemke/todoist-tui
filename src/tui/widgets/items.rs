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
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, StatefulWidget, Widget as RatatuiWidget},
};

#[derive(Debug, Clone)]
pub struct State<'a> {
    pub section_states: Vec<SectionState<'a>>,

    /// `None` indicates that the set of items with no section is the "section" that's currently selected
    pub current_section_id: Option<SectionId>,
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
                KeyCode::Down => current_index.saturating_add(1),
                KeyCode::Up => current_index.saturating_sub(1),
                _ => 0,
            };

            self.current_section_id = self
                .section_states
                .get(next_index)
                .map(|x| x.id.clone())
                .unwrap_or_default();
        }
    }
}

#[derive(Debug, Default)]
pub struct Widget<'a> {
    marker: std::marker::PhantomData<AppState<'a>>,
}

impl<'a> StatefulWidget for Widget<'a> {
    type State = AppState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let constraints =
            Constraint::from_lengths(state.items.section_states.iter().map(SectionState::height));

        let block = Block::default()
            .borders(Borders::ALL)
            // .title(state.projects.selected().clone())
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

        // split the remaining area into pieces
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

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
                        buf,
                        section_state,
                    );
                }
            });
    }
}
