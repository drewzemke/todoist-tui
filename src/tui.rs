use self::{app::App, app_state::Mode};
use crate::{
    cli,
    storage::model_manager::ModelManager,
    sync::{client::Client, Request, ResourceType, Response},
};
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
pub mod app_state;
mod item_input;
mod items_pane;
mod key_hints;
mod projects_pane;
mod section_widget;
mod ui;

/// # Errors
/// Returns an error if something goes wrong during the TUI setup, execution, or teardown.
///
/// # Panics
/// This spawn a thread that can panic if something goes wrong while retreving data from the API.
pub async fn run(model_manager: ModelManager<'_>, client: Result<Client>) -> Result<()> {
    let mut model = model_manager.read_model()?;
    let (sender, receiver) = mpsc::channel::<Response>();
    if let Ok(ref client) = client {
        let client = (*client).clone();
        let commands = model.commands.clone();
        let sync_token = model.sync_token.clone();
        tokio::spawn(async move {
            let request = Request {
                sync_token,
                resource_types: ResourceType::all(),
                commands,
            };

            // FIXME: Need a way (maybe another channel) to communicate to the UI
            // that the sync failed
            let response = client
                .make_request(&request)
                .await
                .expect("Error occurred during full sync.");
            sender
                .send(response)
                .expect("Error occurred while processing server response.");
        });
    }

    let receiver = &receiver;
    let mut app = App::new(&mut model);

    let mut terminal = setup_terminal()?;
    run_main_loop(&mut terminal, &mut app, receiver)?;
    restore_terminal(&mut terminal)?;

    if !model.commands.is_empty() {
        cli::sync(&mut model, &client?, true).await?;
    }
    model_manager.write_model(&model)?;
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
                if app.state.mode == Mode::Exiting {
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
