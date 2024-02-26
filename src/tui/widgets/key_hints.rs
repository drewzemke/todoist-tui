use crate::tui::app_state::{AppState, Mode};
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget, Widget as RatatuiWidget},
};

pub struct KeyHint {
    pub key: String,
    pub action: String,
}

impl KeyHint {
    fn new<S: Into<String>>(key: S, action: S) -> Self {
        Self {
            key: key.into(),
            action: action.into(),
        }
    }

    #[must_use]
    pub fn from_mode(mode: &Mode) -> Vec<KeyHint> {
        match mode {
            Mode::AddingItem => vec![
                Self::new("enter", "add todo"),
                Self::new("escape", "cancel"),
            ],
            Mode::SelectingItems => vec![
                Self::new("q", "quit"),
                Self::new("a", "new todo"),
                Self::new("↑↓", "select"),
                Self::new("space", "mark complete"),
                Self::new("tab", "change focus"),
            ],
            Mode::SelectingProjects => vec![
                Self::new("q", "quit"),
                Self::new("a", "new todo"),
                Self::new("↑↓", "select"),
                Self::new("tab", "change focus"),
            ],
            Mode::Exiting => vec![],
        }
    }
}

impl<'a> From<KeyHint> for Vec<Span<'a>> {
    fn from(hint: KeyHint) -> Self {
        let hint_style = Style::default();
        vec![
            Span::styled(hint.key, hint_style.add_modifier(Modifier::BOLD)),
            Span::styled(": ", hint_style),
            Span::styled(hint.action, hint_style.fg(Color::Gray)),
            Span::raw("  "),
        ]
    }
}

#[derive(Debug, Default)]
pub struct Widget {
    marker: std::marker::PhantomData<AppState>,
}

impl StatefulWidget for Widget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let key_hints = KeyHint::from_mode(&state.mode);
        let key_hint_line: Line = Line::from(
            key_hints
                .into_iter()
                .flat_map(Into::<Vec<Span>>::into)
                .collect::<Vec<Span>>(),
        );
        Paragraph::new(key_hint_line).render(area, buf);
    }
}
