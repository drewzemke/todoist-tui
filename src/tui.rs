use crate::model::Model;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Backend, CrosstermBackend, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io::{self, Stdout};
use tui_input::{backend::crossterm::EventHandler, Input};

/// # Errors
///
/// Returns an error if something goes wrong during the TUI setup, execution, or teardown.
pub fn run(model: &mut Model) -> Result<()> {
    let mut terminal = setup_terminal()?;
    run_main_loop(&mut terminal, model)?;
    restore_terminal(&mut terminal)?;

    Ok(())
}

/// # Errors
///
/// Returns an error if something goes wrong during the TUI setup.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

/// # Errors
///
/// Returns an error if something goes wrong during the TUI teardown.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// # Errors
///
/// Returns an error if something goes wrong during the TUI execution.
fn run_main_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    model: &mut Model,
) -> Result<()> {
    let mut state = AppState {
        mode: Mode::Chillin,
        model,
        input: Input::default(),
    };

    loop {
        render(terminal, &state)?;

        if let Event::Key(key) = event::read()? {
            match state.mode {
                Mode::Chillin => match key.code {
                    KeyCode::Char('a') => {
                        state.mode = Mode::AddingTodo;
                    }
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                },
                Mode::AddingTodo => match key.code {
                    KeyCode::Esc => {
                        state.mode = Mode::Chillin;
                        state.input.reset();
                    }
                    KeyCode::Enter => {
                        state.model.add_item(state.input.value());
                        state.mode = Mode::Chillin;
                        state.input.reset();
                    }
                    _ => {
                        state.input.handle_event(&Event::Key(key));
                    }
                },
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Mode {
    Chillin,
    AddingTodo,
}

pub struct AppState<'a> {
    pub mode: Mode,
    pub model: &'a mut Model,
    pub input: Input,
}

/// # Errors
///
/// Returns an error if something goes wrong during the render process
pub fn render<B: Backend>(terminal: &mut Terminal<B>, state: &AppState) -> Result<()> {
    // turn the list of items in the model into a ratatui list
    let inbox_items: Vec<ListItem> = state
        .model
        .get_inbox_items()
        .iter()
        .map(|item| ListItem::new(&item.content[..]))
        .collect();

    terminal.draw(|frame| {
        let message = Paragraph::new("It's TUI time babyyyyy");
        let inbox_list =
            List::new(inbox_items).block(Block::default().borders(Borders::ALL).title("Inbox"));
        frame.render_widget(message, frame.size());
        frame.render_widget(inbox_list, frame.size());

        let input_rect = centered_rect(frame.size(), 50, 3, Some(2));

        if state.mode == Mode::AddingTodo {
            let input_scroll = state.input.visual_scroll(input_rect.width as usize - 2);
            #[allow(clippy::cast_possible_truncation)]
            let input = Paragraph::new(state.input.value())
                .scroll((0, input_scroll as u16))
                .block(Block::default().title("New Todo").borders(Borders::ALL));
            frame.render_widget(input, input_rect);

            // FIXME: cursor isn't positioned correctly when scrolling
            #[allow(clippy::cast_possible_truncation)]
            frame.set_cursor(
                input_rect.x
                    + (state.input.visual_cursor().max(input_scroll) - input_scroll) as u16
                    + 1,
                input_rect.y + 1,
            );
        }
    })?;
    Ok(())
}

/// Computes a rectangle with a desired width and height that is centered within a container rectangle.
/// The rectangle is constrained to fit within its container and (optionally) be inset by a given margin.
fn centered_rect(
    container: Rect,
    target_width: u16,
    target_height: u16,
    min_margin: Option<u16>,
) -> Rect {
    let min_margin = min_margin.unwrap_or(0);

    let (x, width) = if target_width > container.width - min_margin {
        (container.x + min_margin, container.width - 2 * min_margin)
    } else {
        (
            container.x + (container.width - target_width) / 2,
            target_width,
        )
    };

    let (y, height) = if target_height > container.height - min_margin {
        (container.y + min_margin, container.height - 2 * min_margin)
    } else {
        (
            container.y + (container.height - target_height) / 2,
            target_height,
        )
    };

    Rect {
        x,
        y,
        width,
        height,
    }
}
