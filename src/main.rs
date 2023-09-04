#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
use anyhow::{anyhow, bail, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use todoist::{
    storage::{
        config::{Auth, Manager as ConfigManager},
        data::Manager as ModelManager,
        file::Manager as FileManager,
    },
    sync::{
        self, client::Client, AddItemCommandArgs, CommandArgs, CompleteItemCommandArgs, Item,
        Model, Request,
    },
};
use uuid::Uuid;

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

    let data_dir = if let Some(dir) = args.local_dir.clone() {
        PathBuf::from_str(dir.as_str())?
    } else if let Some(dir) = dirs::data_local_dir() {
        dir.join("tuido")
    } else {
        bail!("Could not find local data directory.");
    };

    let file_manager = FileManager::init(args.local_dir)?;
    let config_manager = ConfigManager::new(&file_manager);
    let model_manager = ModelManager::new(&file_manager);

    match args.command {
        Command::AddTodo { todo, no_sync } => {
            let mut model = model_manager.read_model()?;
            let mut commands = model_manager.read_commands().unwrap_or_default();
            add_item(&todo, &mut model, &mut commands);
            println!("'{todo}' added to inbox.");
            model_manager.write_model(&model)?;
            model_manager.write_commands(&commands)?;
            if !no_sync {
                let api_token = config_manager.read_auth_config()?.api_token;
                let mut sync_data = model_manager.read_model()?;
                let client = Client::new(api_token, args.sync_url);
                incremental_sync(&mut sync_data, &client, &data_dir).await?;
            }
        }
        Command::CompleteTodo { number, no_sync } => {
            let mut model = model_manager.read_model()?;
            let mut commands = model_manager.read_commands().unwrap_or_default();
            let removed_item = complete_item(number, &mut model, &mut commands)?;
            println!("'{}' marked complete.", removed_item.content);
            model_manager.write_model(&model)?;
            model_manager.write_commands(&commands)?;
            if !no_sync {
                let api_token = config_manager.read_auth_config()?.api_token;
                let mut sync_data = model_manager.read_model()?;
                let client = Client::new(api_token, args.sync_url);
                incremental_sync(&mut sync_data, &client, &data_dir).await?;
            }
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
            if incremental {
                let mut model = model_manager.read_model()?;
                incremental_sync(&mut model, &client, &data_dir).await?;
            } else {
                let model = full_sync(&client, &data_dir).await?;
                model_manager.write_model(&model)?;
            }
        }
    };

    Ok(())
}

// create new item
// append to item list in model
// create command
// append to commands list
fn add_item(item: &str, model: &mut Model, commands: &mut Vec<sync::Command>) {
    let inbox_id = &model.user.inbox_project_id;

    // FIXME: should Item.id be a uuid?? probs
    let item_id = Uuid::new_v4();
    let new_item = Item {
        id: item_id.to_string(),
        project_id: inbox_id.clone(),
        content: item.to_owned(),
        checked: false,
    };
    model.items.push(new_item);

    commands.push(sync::Command {
        request_type: "item_add".to_string(),
        temp_id: Some(item_id),
        uuid: Uuid::new_v4(),
        args: CommandArgs::AddItemCommandArgs(AddItemCommandArgs {
            project_id: inbox_id.clone(),
            content: item.to_string(),
        }),
    });
}

fn complete_item<'a>(
    number: usize,
    model: &'a mut Model,
    commands: &mut Vec<sync::Command>,
) -> Result<&'a Item> {
    // look at the current inbox and determine which task is targeted
    let (item_id, num_items) = {
        let inbox_items = model.get_inbox_items();
        let item = inbox_items.get(number - 1).ok_or_else(|| {
            anyhow!(
                "'{number}' is outside of the valid range. Pass a number between 1 and {}.",
                inbox_items.len()
            )
        })?;
        (item.id.clone(), inbox_items.len())
    };

    // update the item's status
    let completed_item = model.complete_item(&item_id).map_err(|_| {
        anyhow!(
            "'{number}' is outside of the valid range. Pass a number between 1 and {num_items}.",
        )
    })?;

    // create a new command and store it
    commands.push(sync::Command {
        request_type: "item_complete".to_owned(),
        temp_id: None,
        uuid: Uuid::new_v4(),
        args: CommandArgs::CompleteItemCommandArgs(CompleteItemCommandArgs { id: item_id }),
    });

    Ok(completed_item)
}

async fn full_sync(client: &Client, data_dir: &PathBuf) -> Result<Model> {
    let commands_file_path = Path::new(data_dir).join("commands.json");
    let mut commands = get_commands(&commands_file_path)?;

    let request_body = Request {
        sync_token: "*".to_string(),
        resource_types: vec!["all".to_string()],
        commands: commands.clone(),
    };

    print!("Syncing... ");
    io::stdout().flush()?;

    let resp = client.make_request(&request_body).await?;
    println!("Done.");

    // update the commands
    resp.temp_id_mapping.iter().for_each(|(temp_id, _)| {
        // remove the matching command
        commands = commands
            .clone()
            .into_iter()
            .filter(
                |sync::Command {
                     temp_id: command_temp_id,
                     ..
                 }| command_temp_id.as_ref() != Some(temp_id),
            )
            .collect();
    });

    let model: Model = resp.try_into()?;

    // update the commands file
    fs::write(commands_file_path, serde_json::to_string_pretty(&commands)?)?;

    Ok(model)
}

async fn incremental_sync(
    sync_data: &mut Model,
    client: &Client,
    data_dir: &PathBuf,
) -> Result<()> {
    // get commands that we need to send
    let commands_file_path = Path::new(data_dir).join("commands.json");
    let mut commands = get_commands(&commands_file_path)?;

    let request_body = Request {
        sync_token: sync_data.sync_token.clone(),
        resource_types: vec!["all".to_string()],
        // HACK: no clone here plz
        commands: commands.clone(),
    };

    print!("Syncing... ");
    io::stdout().flush()?;

    let resp = client.make_request(&request_body).await?;
    println!("Done.");

    // update the sync_data with the result
    sync_data.sync_token = resp.sync_token;
    resp.temp_id_mapping.iter().for_each(|(temp_id, real_id)| {
        // HACK: should we do something else if we don't find a match?
        if let Some(matching_item) = sync_data
            .items
            .iter_mut()
            .find(|item| item.id == temp_id.to_string())
        {
            matching_item.id = real_id.clone();
        }

        // remove the matching command
        commands = commands
            .clone()
            .into_iter()
            .filter(
                |sync::Command {
                     temp_id: command_temp_id,
                     ..
                 }| command_temp_id.as_ref() != Some(temp_id),
            )
            .collect();
    });
    if let Some(user) = resp.user {
        sync_data.user = user;
    }

    let sync_storage_path = Path::new(data_dir).join("sync.json");

    // store in file
    fs::create_dir_all(Path::new(data_dir).join("data"))?;
    let file = fs::File::create(sync_storage_path)?;
    serde_json::to_writer_pretty(file, &sync_data)?;

    // update the commands file
    fs::write(commands_file_path, serde_json::to_string_pretty(&commands)?)?;

    Ok(())
}

fn get_commands(commands_file_path: &PathBuf) -> Result<Vec<sync::Command>> {
    let commands: Vec<sync::Command> = if commands_file_path.exists() {
        let file = fs::read_to_string(commands_file_path)?;
        serde_json::from_str::<Vec<sync::Command>>(&file)?
    } else {
        Vec::new()
    };
    Ok(commands)
}
