#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
use std::sync::mpsc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tod::{
    cli,
    model::item::Item,
    storage::{
        config_manager::{Auth, ConfigManager},
        file_manager::FileManager,
        model_manager::ModelManager,
    },
    sync::{client::Client, Request, ResourceType, Response},
    tui,
};

#[derive(Parser)]
#[command(author)]
struct Args {
    #[command(subcommand)]
    command: Option<CliCommand>,

    /// Override the URL for the Todoist Sync API (mostly for testing purposes)
    #[arg(long = "sync-url", hide = true)]
    sync_url: Option<String>,

    /// Override the local app storage directory (mostly for testing purposes)
    #[arg(long = "local-dir", hide = true)]
    local_dir: Option<String>,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Add a new todo to your inbox
    #[command(name = "add")]
    AddTodo {
        /// The text of the todo
        todo: String,

        /// Don't sync data with the server
        #[arg(long = "no-sync", short)]
        no_sync: bool,
    },

    /// Mark a todo in the inbox complete
    #[command(name = "complete")]
    CompleteTodo {
        /// The number of the todo that's displayed with the `list` command
        number: usize,

        /// Don't sync data with the server
        #[arg(long = "no-sync", short)]
        no_sync: bool,
    },

    /// List the items in your inbox
    #[command(name = "list")]
    ListInbox,

    /// Store a Todoist API token
    #[command(name = "set-token")]
    SetApiToken {
        /// The Todoist API token
        token: String,
    },

    /// Sync data with the Todoist server
    #[command()]
    Sync {
        /// Only sync changes made locally since the last full sync
        #[arg(long, short)]
        incremental: bool,
    },
}

#[derive(Serialize, Deserialize)]
struct Config {
    api_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let file_manager = FileManager::init(args.local_dir)?;
    let config_manager = ConfigManager::new(&file_manager);
    let model_manager = ModelManager::new(&file_manager);

    let client = config_manager
        .get_api_token()
        .map(|token| Client::new(token, args.sync_url));

    match args.command {
        None => {
            let mut model = model_manager.read_model()?;

            // kick off a full sync that sends its results to the tui app
            let (sender, receiver) = mpsc::channel::<Response>();
            if let Ok(ref client) = client {
                let client = client.clone();
                let commands = model.commands.clone();
                tokio::spawn(async move {
                    let request = Request {
                        sync_token: "*".into(),
                        resource_types: vec![ResourceType::All],
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

            tui::run(&mut model, &receiver)?;

            if !model.commands.is_empty() {
                cli::sync(&mut model, &client?, true).await?;
            }
            model_manager.write_model(&model)?;
        }

        Some(command) => match command {
            CliCommand::AddTodo { todo, no_sync } => {
                let mut model = model_manager.read_model()?;
                model.add_item(&todo);
                println!("'{todo}' added to inbox.");
                if !no_sync {
                    cli::sync(&mut model, &client?, true).await?;
                }
                model_manager.write_model(&model)?;
            }

            CliCommand::CompleteTodo { number, no_sync } => {
                let mut model = model_manager.read_model()?;
                cli::complete_item(number, &mut model)?;
                if !no_sync {
                    cli::sync(&mut model, &client?, true).await?;
                }
                model_manager.write_model(&model)?;
            }

            CliCommand::ListInbox => {
                let model = model_manager.read_model()?;
                let inbox_items = model.get_inbox_items(true);

                println!("Inbox: ");
                for (index, Item { content, .. }) in inbox_items.iter().enumerate() {
                    println!("[{}] {content}", index + 1);
                }
            }

            CliCommand::SetApiToken { token } => {
                config_manager.write_auth_config(&Auth { api_token: token })?;
                println!("Stored API token.");
            }

            CliCommand::Sync { incremental } => {
                let mut model = model_manager.read_model()?;
                cli::sync(&mut model, &client?, incremental).await?;
                model_manager.write_model(&model)?;
            }
        },
    }

    Ok(())
}
