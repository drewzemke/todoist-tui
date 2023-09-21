use self::app::App;
use crate::model::Model;
use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

pub mod app;
pub mod ui;

/// # Errors
/// Returns an error if something goes wrong during the TUI setup, execution, or teardown.
pub fn run(model: &mut Model) -> Result<()> {
    let mut terminal = setup_terminal()?;
    run_main_loop(&mut terminal, model)?;
    restore_terminal(&mut terminal)?;

    Ok(())
}

/// # Errors
/// Returns an error if something goes wrong during the TUI setup.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

/// # Errors
/// Returns an error if something goes wrong during the TUI teardown.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// # Errors
/// Returns an error if something goes wrong during the TUI execution.
fn run_main_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    model: &mut Model,
) -> Result<()> {
    let mut app = App::new(model);

    loop {
        // render
        terminal.draw(|frame| {
            app.render(frame);
        })?;

        // process input
        if let Event::Key(key) = event::read()? {
            if app.handle_key(key) {
                return Ok(());
            }
        }
    }
}
