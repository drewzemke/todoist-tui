use super::section_widget::{SectionWidget, State as SectionWidgetState};
use crate::model::{
    item::{Id as ItemId, Item},
    project::Project,
    section::{Id as SectionId, Section},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, StatefulWidget, Widget},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ItemTree<'a> {
    section_widgets: Vec<(Option<SectionId>, SectionWidget<'a>)>,
}

impl<'a> ItemTree<'a> {
    pub fn new(items: &'_ [Item], sections: &'_ [Section], project: &Project) -> Self {
        let mut section_widgets: Vec<_> = sections
            .iter()
            .filter(|section| section.project_id == project.id)
            .map(|section| {
                let items_in_section: Vec<_> = items
                    .iter()
                    .filter(|item| item.section_id.as_ref().is_some_and(|id| id == &section.id))
                    .collect();
                let section_widget =
                    SectionWidget::new(Some(section.name.clone()), &items_in_section);
                (Some(section), section_widget)
            })
            .collect();

        let items_in_no_section: Vec<_> = items
            .iter()
            .filter(|item| item.project_id == project.id)
            .filter(|item| item.section_id.is_none())
            .collect();
        let section_widget = SectionWidget::new(None, &items_in_no_section);
        section_widgets.push((None, section_widget));

        section_widgets.sort_unstable_by(|(section_a, _), (section_b, _)| section_a.cmp(section_b));

        let section_widgets = section_widgets
            .into_iter()
            .map(|(section, widget)| (section.map(|s| s.id.clone()), widget))
            .collect();

        Self { section_widgets }
    }
}

impl<'a> StatefulWidget for ItemTree<'a> {
    type State = ItemTreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let constraints =
            Constraint::from_lengths(self.section_widgets.iter().map(|(section_id, widget)| {
                state
                    .section_states
                    .get(section_id)
                    .map_or(4, |state| widget.height(state))
            }));

        let block = Block::default()
            .borders(Borders::ALL)
            .title(state.project_name.clone())
            .border_style(Style::default().fg(if state.focused {
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
        self.section_widgets
            .iter()
            .enumerate()
            .for_each(|(index, (section_id, widget))| {
                // FIXME (maybe) : this just doesn't render stuff if it can't find the layout or
                // state -- is that good? (better than crashing I guess)
                if let Some(rect) = layout.get(index) {
                    if let Some(state) = state.section_states.get_mut(section_id) {
                        widget.clone().render(*rect, buf, state);
                    }
                }
            });
    }
}

#[derive(Debug, Clone)]
pub struct ItemTreeState {
    project_name: String,
    section_states: HashMap<Option<SectionId>, SectionWidgetState>,
    current_section: Option<SectionId>,
    focused: bool,
}

impl ItemTreeState {
    pub fn new(items: &'_ [Item], sections: &'_ [Section], project: &Project) -> Self {
        // FIXME: dry this up a bit?
        let mut section_states: Vec<_> = sections
            .iter()
            .filter(|section| section.project_id == project.id)
            .map(|section| {
                let items_in_section: Vec<_> = items
                    .iter()
                    .filter(|item| item.section_id.as_ref().is_some_and(|id| id == &section.id))
                    .collect();
                let mut state = SectionWidgetState::default();
                for item in items_in_section {
                    if !item.collapsed {
                        state.open(vec![item.id.clone()]);
                    }
                }
                (Some(section), state)
            })
            .collect();

        let items_in_no_section: Vec<_> = items
            .iter()
            .filter(|item| item.project_id == project.id)
            .filter(|item| item.section_id.is_none())
            .collect();
        let mut state = SectionWidgetState::default();
        for item in items_in_no_section {
            if !item.collapsed {
                state.open(vec![item.id.clone()]);
            }
        }
        section_states.push((None, state));

        let section_states = section_states
            .into_iter()
            .map(|(section, widget)| (section.map(|s| s.id.clone()), widget))
            .collect();

        Self {
            section_states,
            current_section: None,
            project_name: project.name.clone(),
            focused: false,
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn current_state(&self) -> Option<&SectionWidgetState> {
        self.section_states.get(&self.current_section)
    }

    fn current_state_mut(&mut self) -> Option<&mut SectionWidgetState> {
        self.section_states.get_mut(&self.current_section)
    }

    pub fn selected(&self) -> Option<ItemId> {
        let state = self.current_state()?;
        state.selected().into_iter().last()
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('\n' | ' ') => {
                if let Some(state) = self.current_state_mut() {
                    state.toggle_selected();
                }
            }
            KeyCode::Left => {
                if let Some(state) = self.current_state_mut() {
                    state.key_left();
                }
            }
            KeyCode::Right => {
                if let Some(state) = self.current_state_mut() {
                    state.key_right();
                }
            }
            // FIXME
            // KeyCode::Down => self.state.key_down(&self.tree_items),
            // KeyCode::Up => self.state.key_up(&self.tree_items),
            _ => {}
        }
    }
}
