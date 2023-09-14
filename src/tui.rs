use crate::model::Model;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Backend, CrosstermBackend},
    widgets::Paragraph,
    Terminal,
};
use std::{
    io::{self, Stdout},
    time::Duration,
};

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
    loop {
        render(terminal)?;
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if KeyCode::Char('q') == key.code {
                    break;
                }
            }
        }
    }

    Ok(())
}

/// # Errors
///
/// Returns an error if something goes wrong during the render process
pub fn render<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    terminal.draw(|frame| {
        let message = Paragraph::new("It's TUI time babyyyyy");
        frame.render_widget(message, frame.size());
    })?;
    Ok(())
}
