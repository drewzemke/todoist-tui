#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
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
    sync::client::Client,
};

#[derive(Parser)]
#[command(author)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Override the URL for the Todoist Sync API (mostly for testing purposes)
    #[arg(long = "sync-url", hide = true)]
    sync_url: Option<String>,

    /// Override the local app storage directory (mostly for testing purposes)
    #[arg(long = "local-dir", hide = true)]
    local_dir: Option<String>,
}

#[derive(Subcommand)]
enum Command {
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

    match args.command {
        Command::AddTodo { todo, no_sync } => {
            let mut model = model_manager.read_model()?;
            model.add_item(&todo);
            println!("'{todo}' added to inbox.");
            if !no_sync {
                let api_token = config_manager.read_auth_config()?.api_token;
                let client = Client::new(api_token, args.sync_url);
                cli::sync(&mut model, &client, true).await?;
            }
            model_manager.write_model(&model)?;
        }

        Command::CompleteTodo { number, no_sync } => {
            let mut model = model_manager.read_model()?;
            let removed_item = cli::complete_item(number, &mut model)?;
            println!("'{}' marked complete.", removed_item.content);
            if !no_sync {
                let api_token = config_manager.read_auth_config()?.api_token;
                let client = Client::new(api_token, args.sync_url);
                cli::sync(&mut model, &client, true).await?;
            }
            model_manager.write_model(&model)?;
        }

        Command::ListInbox => {
            let model = model_manager.read_model()?;
            let inbox_items = model.get_inbox_items();

            println!("Inbox: ");
            for (index, Item { content, .. }) in inbox_items.iter().enumerate() {
                println!("[{}] {content}", index + 1);
            }
        }

        Command::SetApiToken { token } => {
            config_manager.write_auth_config(&Auth { api_token: token })?;
            println!("Stored API token.");
        }

        Command::Sync { incremental } => {
            let api_token = config_manager.read_auth_config()?.api_token;
            let client = Client::new(api_token, args.sync_url);
            let mut model = model_manager.read_model()?;
            cli::sync(&mut model, &client, incremental).await?;
            model_manager.write_model(&model)?;
        }
    };

    Ok(())
}
