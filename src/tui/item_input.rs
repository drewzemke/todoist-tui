use crate::model::due_date::Due;
use chrono::NaiveDate;
use crossterm::event::Event;
use ratatui::{
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Debug, Default, Clone)]
pub struct ItemInput {
    input: Input,
    today: NaiveDate,
}

impl ItemInput {
    pub fn new(today: NaiveDate) -> Self {
        Self {
            today,
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        self.input.reset();
    }

    pub fn handle_event(&mut self, event: &Event) {
        self.input.handle_event(event);
    }

    pub fn cursor_position(&self, area: Rect) -> (u16, u16) {
        let input_scroll = self.input.visual_scroll(area.width as usize - 2);
        #[allow(clippy::cast_possible_truncation)]
        (
            area.x + (self.input.visual_cursor().max(input_scroll) - input_scroll) as u16 + 1,
            area.y + 1,
        )
    }

    pub fn get_new_item(&self) -> (String, Option<Due>) {
        // process the input to maybe find a date
        let input = self.input.value();
        let due_date = Due::parse_from_str(input, self.today);

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

        let due = due_date.map(|(date, _)| date);
        (content, due)
    }
}

impl Widget for ItemInput {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        // preprocess the current input string to see if there's a date inside
        let due_date = Due::parse_from_str(self.input.value(), self.today);
        let input_widget = if let Some((_, range)) = due_date {
            let (before, after) = self.input.value().split_at(range.start);
            let (date, after) = after.split_at(range.end - before.len());
            let line = Line::from(vec![
                Span::styled(before, Style::default().fg(Color::White)),
                Span::styled(
                    date,
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(after, Style::default().fg(Color::White)),
            ]);
            Paragraph::new(line)
        } else {
            Paragraph::new(self.input.value())
        };

        // figure the right amount to scroll the input by
        let input_scroll = self.input.visual_scroll(area.width as usize - 2);
        #[allow(clippy::cast_possible_truncation)]
        let input_widget = input_widget.scroll((0, input_scroll as u16)).block(
            Block::default()
                .title("New Todo")
                .border_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL),
        );

        Clear.render(area, buf);
        input_widget.render(area, buf);
    }
}
