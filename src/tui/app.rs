use super::{
    key_hints::KeyHint,
    lists::{item_list, project_list, State as ListState},
    ui::centered_rect,
};
use crate::model::{
    item::{Due, DueDate, Item},
    project::Project,
    Model,
};
use chrono::{Local, NaiveDate};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use smart_date::{FlexibleDate, Parsed};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AddingItem,
    SelectingItems,
    SelectingProjects,
    Exiting,
}

/// Manages the UI state and data model
pub struct App<'a> {
    pub mode: Mode,
    pub model: &'a mut Model,
    input: Input,
    item_list_state: ListState,
    project_list_state: ListState,
    today: NaiveDate,
}

impl<'a> App<'a> {
    pub fn new(model: &'a mut Model) -> Self {
        let num_items = model.get_inbox_items(false).len();
        let num_projects = model.projects.len();
        Self {
            mode: Mode::SelectingItems,
            model,
            input: Input::default(),
            item_list_state: ListState::with_length(num_items),
            project_list_state: ListState::new(num_projects, 0),
            today: Local::now().date_naive(),
        }
    }

    /// FIXME: This is used for testing; is there a more elegant way to accomplish that?
    pub fn new_with_date(model: &'a mut Model, today: NaiveDate) -> Self {
        Self {
            today,
            ..Self::new(model)
        }
    }

    /// Updates the inner state of model after the model changes.
    pub fn update_state(&mut self) {
        let num_items = self.items_in_selected_project().len();
        self.item_list_state.set_length(num_items);
    }

    fn selected_project(&self) -> Option<&Project> {
        let projects = self.model.projects();
        self.project_list_state
            .selected_index()
            .map(|index| projects[index])
    }

    fn items_in_selected_project(&self) -> Vec<&Item> {
        let projects = self.model.projects();
        self.project_list_state
            .selected_index()
            .map(|index| projects[index])
            .map(|project| self.model.get_items_in_project(&project.id))
            .unwrap_or_default()
    }

    /// Manages how the whole app reacts to an individual user keypress.
    pub fn handle_key(&mut self, key: event::KeyEvent) {
        match self.mode {
            Mode::SelectingItems => match key.code {
                KeyCode::Char('a') => {
                    self.mode = Mode::AddingItem;
                }
                KeyCode::Char('q') => {
                    self.mode = Mode::Exiting;
                }
                KeyCode::Up | KeyCode::Down => self.item_list_state.handle_key(key),
                KeyCode::Tab | KeyCode::Left => {
                    self.mode = Mode::SelectingProjects;
                }
                KeyCode::Char(' ') => {
                    if let Some(selected_index) = self.item_list_state.selected_index() {
                        let items = self.items_in_selected_project();
                        let item = items[selected_index];
                        self.model.mark_item(&item.id.clone(), !item.checked);
                        self.update_state();
                    }
                }
                _ => {}
            },
            Mode::SelectingProjects => match key.code {
                KeyCode::Char('a') => {
                    self.mode = Mode::AddingItem;
                }
                KeyCode::Char('q') => {
                    self.mode = Mode::Exiting;
                }
                KeyCode::Up | KeyCode::Down => {
                    self.project_list_state.handle_key(key);
                    self.update_state();
                }
                KeyCode::Tab | KeyCode::Right => {
                    self.mode = Mode::SelectingItems;
                }
                _ => {}
            },
            Mode::AddingItem => match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::SelectingItems;
                    self.input.reset();
                }
                KeyCode::Enter => {
                    let selected_project = self.selected_project();
                    if let Some(selected_project) = selected_project {
                        let project_id = selected_project.id.clone();

                        // process the input to maybe find a date
                        // TODO : dep inj for today's date
                        let input = self.input.value();
                        let due_date = FlexibleDate::find_and_parse_in_str(input)
                            .map(|Parsed { data, range }| (data.into_naive_date(self.today), range))
                            .map(|(date, range)| {
                                (
                                    Due {
                                        date: DueDate::Date(date),
                                    },
                                    range,
                                )
                            });

                        // if a date was found, remove the matched string from the text content of the new item
                        let content = if let Some((_, ref range)) = due_date {
                            format!(
                                "{}{}",
                                &input[0..range.start],
                                &input[input.len().min(range.end + 1)..input.len()]
                            )
                        } else {
                            input.to_string()
                        };

                        self.model.add_item(
                            content.trim(),
                            project_id,
                            due_date.map(|(date, _)| date),
                        );
                        self.update_state();
                        self.mode = Mode::SelectingItems;
                        self.input.reset();
                    }
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
    pub fn render<'b, B: Backend + 'b>(&mut self, frame: &mut Frame<'b, B>) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Max(1)])
            .split(frame.size());
        let main_panel = layout[0];
        let bottom_panel = layout[1];

        let main_panel_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_panel);
        let main_left = main_panel_layout[0];
        let main_right = main_panel_layout[1];

        // render the project list
        let projects = self.model.projects();
        let project_list = project_list(&projects, self.mode == Mode::SelectingProjects);
        frame.render_stateful_widget(project_list, main_left, &mut self.project_list_state.state);

        // render the item list
        let selected_project = self
            .project_list_state
            .selected_index()
            // TODO: use `get` here
            .map(|index| projects[index]);
        if let Some(selected_project) = selected_project {
            let items = self.model.get_items_in_project(&selected_project.id);
            let item_list = item_list(
                &items,
                &selected_project.name,
                self.mode == Mode::SelectingItems,
            );
            frame.render_stateful_widget(item_list, main_right, &mut self.item_list_state.state);
        }

        // render the key hints
        let key_hints = KeyHint::from_mode(&self.mode);
        let key_hint_line: Line = Line::from(
            key_hints
                .into_iter()
                .flat_map(Into::<Vec<Span>>::into)
                .collect::<Vec<Span>>(),
        );

        frame.render_widget(Paragraph::new(key_hint_line), bottom_panel);

        if self.mode == Mode::AddingItem {
            let input_rect = centered_rect(frame.size(), 50, 3, Some(2));
            let input_scroll = self.input.visual_scroll(input_rect.width as usize - 2);
            #[allow(clippy::cast_possible_truncation)]
            let input = Paragraph::new(self.input.value())
                .scroll((0, input_scroll as u16))
                .block(Block::default().title("New Todo").borders(Borders::ALL));
            frame.render_widget(Clear, input_rect);
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
