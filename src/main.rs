#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use todoist::sync::{self, AddItemCommand, AddItemRequestArgs, Item, Request, Response};
use uuid::Uuid;

// TODO: make some of these into commands rather than optional arguments
#[derive(Parser)]
#[command(author)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Override the URL for the Todoist Sync API (mostly for testing purposes).
    #[arg(long = "sync-url", hide = true)]
    sync_url: Option<String>,

    /// Override the local app storage directory (mostly for testing purposes).
    #[arg(long = "local-dir", hide = true)]
    local_dir: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Add a new todo to your inbox.
    #[command(name = "add")]
    AddTodo {
        /// The text of the todo.
        todo: String,
    },

    /// List the items in your inbox.
    #[command(name = "list")]
    ListInbox,

    /// Store a Todoist API token.
    #[command(name = "set-token")]
    SetApiToken {
        /// The Todoist API token.
        token: String,
    },

    /// Sync data with the Todoist server.
    #[command()]
    Sync,
}

#[derive(Serialize, Deserialize)]
struct Config {
    api_token: String,
}

const SYNC_URL: &str = "https://api.todoist.com/sync/v9";
const MISSING_API_TOKEN_MESSAGE : &str = "Could not find an API token. Go to https://todoist.com/app/settings/integrations/developer to get yours, then re-run with '--set-api-token <TOKEN>'.";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let sync_url = args.sync_url.unwrap_or(SYNC_URL.into());

    let data_dir = if let Some(dir) = args.local_dir {
        PathBuf::from_str(dir.as_str())?
    } else if let Some(dir) = dirs::data_local_dir() {
        dir.join("tuido")
    } else {
        return Err("Could not find local data directory.".into());
    };

    match args.command {
        Command::AddTodo { todo } => {
            // FIXME: probably want to split up the network/file responsibilities here
            add_item(&data_dir, &todo)?;

            println!("Todo '{todo}' added to inbox.");
        }
        Command::ListInbox => {
            let inbox_items = get_inbox_items(&data_dir)?;

            println!("Inbox: ");
            for Item { content, .. } in inbox_items {
                println!("- {content}");
            }
        }
        Command::SetApiToken { token } => set_api_token(token, &data_dir)?,
        Command::Sync => {
            let api_token = get_api_token(&data_dir).map_err(|_e| MISSING_API_TOKEN_MESSAGE)?;
            full_sync(&sync_url, &api_token, &data_dir).await?;
        }
    };

    println!("Bye!");
    Ok(())
}

fn get_api_token(data_dir: &PathBuf) -> Result<String, Box<dyn Error>> {
    let auth_file_name = "client_auth.toml";
    let auth_path = Path::new(data_dir).join(auth_file_name);
    let file = fs::read_to_string(auth_path)?;
    let config: Config = toml::from_str(file.as_str())?;

    Ok(config.api_token)
}

fn set_api_token(api_token: String, data_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let auth_file_name = "client_auth.toml";
    let auth_path = Path::new(data_dir).join(auth_file_name);
    fs::write(&auth_path, toml::to_string_pretty(&Config { api_token })?)?;
    println!("Stored API token in '{}'.", auth_path.display());
    Ok(())
}

fn add_item(data_dir: &PathBuf, item: &str) -> Result<(), Box<dyn Error>> {
    // read in the stored data
    let sync_file_path = Path::new(data_dir).join("data").join("sync.json");

    let file = fs::read_to_string(sync_file_path)?;
    let mut data = serde_json::from_str::<Response>(&file)?;

    // create a new item and add it to the item list
    let inbox_id = &data
        .user
        .as_ref()
        .ok_or(std::convert::Into::<Box<dyn Error>>::into(
            "Could not find inbox project id in stored data.",
        ))?
        .inbox_project_id;

    // FIXME: should Item.id be a uuid?? probs
    let item_id = Uuid::new_v4();
    let new_item = Item {
        id: item_id.to_string(),
        project_id: inbox_id.clone(),
        content: item.to_owned(),
    };
    data.items.push(new_item);

    // store the data
    let sync_storage_path = Path::new(data_dir).join("data").join("sync.json");
    let file = fs::File::create(sync_storage_path)?;
    serde_json::to_writer_pretty(file, &data)?;

    // create a new command and store it
    let commands_file_path = Path::new(data_dir).join("data").join("commands.json");

    let mut commands: Vec<sync::Command> = if commands_file_path.exists() {
        let file = fs::read_to_string(&commands_file_path)?;
        serde_json::from_str::<Vec<sync::Command>>(&file)?
    } else {
        Vec::new()
    };

    commands.push(sync::Command::AddItem(AddItemCommand {
        request_type: "add_item".to_owned(),
        temp_id: item_id,
        uuid: Uuid::new_v4(),
        args: AddItemRequestArgs {
            project_id: inbox_id.clone(),
            content: item.to_owned(),
        },
    }));

    fs::write(commands_file_path, serde_json::to_string_pretty(&commands)?)?;

    Ok(())
}

fn get_inbox_items(data_dir: &PathBuf) -> Result<Vec<Item>, Box<dyn Error>> {
    // read in the stored data
    let sync_file_path = Path::new(data_dir).join("data").join("sync.json");

    let file = fs::read_to_string(sync_file_path)?;
    // HACK: wrong type, need a common storage type
    let data = serde_json::from_str::<Response>(&file)?;

    // get the items with the correct id
    if let Some(inbox_id) = data.user.map(|user| user.inbox_project_id) {
        let items: Vec<Item> = data
            .items
            .into_iter()
            .filter(|item| item.project_id == inbox_id)
            .collect();
        Ok(items)
    } else {
        Err("Could not find inbox project id in stored data.".into())
    }
}

async fn full_sync(
    sync_url: &String,
    api_token: &String,
    data_dir: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let request_body = Request {
        sync_token: "*".to_string(),
        resource_types: vec!["all".to_string()],
        commands: vec![],
    };

    print!("Performing a full sync... ");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{sync_url}/sync"))
        .header("Authorization", format!("Bearer {api_token}"))
        .json(&request_body)
        .send()
        .await
        .map(reqwest::Response::json::<sync::Response>)?
        .await?;
    println!("Done.");

    let sync_storage_path = Path::new(data_dir).join("data").join("sync.json");

    // store in file
    fs::create_dir_all(Path::new(data_dir).join("data"))?;
    let file = fs::File::create(&sync_storage_path)?;
    serde_json::to_writer_pretty(file, &resp)?;
    println!("Stored sync data in '{}'.", sync_storage_path.display());

    Ok(())
}
