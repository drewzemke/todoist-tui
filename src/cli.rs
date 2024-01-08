use crate::{
    model::{due_date::Due, item::Item, Model},
    storage::{
        config_manager::{Auth, ConfigManager},
        model_manager::ModelManager,
    },
    sync::{client::Client, Request, ResourceType},
};
use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDateTime};
use clap::{Parser, Subcommand};
use std::io::{self, Write};

#[derive(Parser, Clone)]
#[command(author)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Override the URL for the Todoist Sync API (mostly for testing purposes)
    #[arg(long = "sync-url-override", hide = true)]
    pub sync_url_override: Option<String>,

    /// Override the local app storage directory (mostly for testing purposes)
    #[arg(long = "local-dir-override", hide = true)]
    pub local_dir_override: Option<String>,

    /// Override the date/time the app uses as current date/time
    #[arg(long = "date-time-override", hide = true)]
    pub datetime_override: Option<NaiveDateTime>,
}

#[derive(Subcommand, Clone)]
pub enum Command {
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

/// # Errors
///
/// Returns an error if `number` does not correspond to a valid item
pub fn complete_item(number: usize, model: &mut Model) -> Result<()> {
    // look at the current inbox and determine which task is targeted
    let inbox_items = model.get_inbox_items(true);
    let num_items = inbox_items.len();

    let item = inbox_items.get(number - 1).ok_or_else(|| {
        anyhow!(
            "'{number}' is outside of the valid range. Pass a number between 1 and {num_items}.",
        )
    })?;
    let content = item.content.clone();

    model.mark_item(&item.id.clone(), true);
    println!("'{content}' marked complete.");

    Ok(())
}

/// # Errors
///
/// Returns an error if something goes awry while processing the command.
pub async fn handle_command(
    command: &Command,
    args: Args,
    model_manager: ModelManager<'_>,
    client: Result<Client>,
    config_manager: ConfigManager<'_>,
) -> Result<()> {
    match command {
        Command::AddTodo { todo, no_sync, due } => {
            // TODO: parse the date first, it might be no good and we'll need to error out
            let today = args
                .datetime_override
                .unwrap_or(Local::now().naive_local())
                .date();
            let due_date = due.as_ref().and_then(|due| {
                Due::parse_from_str(due, today).and_then(|(date, range)| {
                    // reject the due date if it didn't parse exactly
                    if range == (0..due.len()) {
                        Some(date)
                    } else {
                        None
                    }
                })
            });

            let mut model = model_manager.read_model()?;
            model.add_item_to_inbox(todo, due_date);
            println!("'{todo}' added to inbox.");
            if !no_sync {
                sync(&mut model, &client?, true).await?;
            }
            model_manager.write_model(&model)?;
        }

        Command::CompleteTodo { number, no_sync } => {
            let mut model = model_manager.read_model()?;
            complete_item(*number, &mut model)?;
            if !no_sync {
                sync(&mut model, &client?, true).await?;
            }
            model_manager.write_model(&model)?;
        }

        Command::ListInbox => {
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

        Command::SetApiToken { token } => {
            config_manager.write_auth_config(&Auth {
                api_token: token.clone(),
            })?;
            println!("Stored API token.");
        }

        Command::Sync { incremental } => {
            let mut model = model_manager.read_model()?;
            sync(&mut model, &client?, *incremental).await?;
            model_manager.write_model(&model)?;
        }
    };

    Ok(())
}

// FIXME: this probably isn't the right place for this function
/// # Errors
///
/// Returns an error if something goes wrong while sending/receiving data from the Todoist API.
pub async fn sync(model: &mut Model, client: &Client, incremental: bool) -> Result<()> {
    let sync_token = if incremental {
        model.sync_token.clone()
    } else {
        "*".to_string()
    };

    let request = Request {
        sync_token,
        resource_types: ResourceType::all(),
        commands: model.commands.clone(),
    };

    print!("Syncing... ");
    io::stdout().flush()?;

    let response = client.make_request(&request).await?;
    println!("Done.");

    // update the sync_data with the result
    model.update(response);

    Ok(())
}
