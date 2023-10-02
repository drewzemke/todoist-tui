use self::app::{App, Mode};
use crate::{model::Model, sync::Response};
use anyhow::Result;
use crossterm::{
    event::{self, poll, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    sync::mpsc,
    time::Duration,
};

pub mod app;
pub mod lists;
pub mod ui;

/// # Errors
/// Returns an error if something goes wrong during the TUI setup, execution, or teardown.
pub fn run(model: &mut Model, receiver: &mpsc::Receiver<Response>) -> Result<()> {
    let mut app = App::new(model);

    let mut terminal = setup_terminal()?;
    run_main_loop(&mut terminal, &mut app, receiver)?;
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
    app: &mut App<'_>,
    receiver: &mpsc::Receiver<Response>,
) -> Result<()> {
    loop {
        // render
        terminal.draw(|frame| {
            app.render(frame);
        })?;

        if poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
                if app.mode == Mode::Exiting {
                    return Ok(());
                }
            }
        } else {
            // check if the receiver received anything
            if let Ok(response) = receiver.try_recv() {
                app.model.update(response);
                app.update_state();
            }
        } // process input
    }
}
