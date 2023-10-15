#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
use std::sync::mpsc;

use anyhow::Result;
use chrono::{Local, NaiveDateTime};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tod::{
    cli,
    model::item::{Due, DueDate, Item},
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
    #[arg(long = "sync-url-override", hide = true)]
    sync_url_override: Option<String>,

    /// Override the local app storage directory (mostly for testing purposes)
    #[arg(long = "local-dir-override", hide = true)]
    local_dir_override: Option<String>,

    /// Override the date/time the app uses as current date/time
    #[arg(long = "date-time-override", hide = true)]
    datetime_override: Option<NaiveDateTime>,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Add a new todo to your inbox
    #[command(name = "add")]
    AddTodo {
        /// The text of the todo
        todo: String,

        /// When the todo is due
        #[arg(long, short)]
        due: Option<String>,

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

    let file_manager = FileManager::init(args.local_dir_override)?;
    let config_manager = ConfigManager::new(&file_manager);
    let model_manager = ModelManager::new(&file_manager);

    let client = config_manager
        .get_api_token()
        .map(|token| Client::new(token, args.sync_url_override));

    match args.command {
        None => {
            let mut model = model_manager.read_model()?;

            // kick off a full sync that sends its results to the tui app
            let (sender, receiver) = mpsc::channel::<Response>();
            if let Ok(ref client) = client {
                let client = client.clone();
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

            tui::run(&mut model, &receiver)?;

            if !model.commands.is_empty() {
                cli::sync(&mut model, &client?, true).await?;
            }
            model_manager.write_model(&model)?;
        }

        Some(command) => match command {
            CliCommand::AddTodo { todo, no_sync, due } => {
                // TODO: parse the date first, it might be no good and we'll need to error out
                let due_date = due
                    .and_then(|date_string| {
                        let now = args.datetime_override.map_or(Local::now(), |datetime| {
                            // FIXME: stop using <Local> in `smart-date`, so we can remove this unwrap
                            // (it shouldn't ever fail afaik)
                            datetime.and_local_timezone(Local).unwrap()
                        });
                        smart_date::parse(&date_string, &now)
                    })
                    .map(|result| Due {
                        date: DueDate::DateTime(result.data.naive_local()),
                    });

                let mut model = model_manager.read_model()?;
                model.add_item_to_inbox(&todo, due_date);
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

                if inbox_items.is_empty() {
                    println!("Your inbox is empty.");
                } else {
                    println!("Inbox: ");
                    for (index, Item { content, .. }) in inbox_items.iter().enumerate() {
                        println!("[{}] {content}", index + 1);
                    }
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
