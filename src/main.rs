#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
use clap::Parser;
use serde::Deserialize;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use todoist::sync::{
    AddItemCommand, AddItemRequest, AddItemRequestArgs, GetUserRequest, Response, User,
};
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(author)]
struct Args {
    /// Add a new todo to the inbox.
    #[arg(short, long = "add", name = "TODO")]
    add_todo: Option<String>,

    /// Override the URL for the Todoist Sync API (mostly for testing purposes).
    #[arg(long = "sync-url", hide = true)]
    sync_url: Option<String>,

    /// Override the local app storage directory (mostly for testing purposes).
    #[arg(long = "local-dir", hide = true)]
    local_dir: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    api_key: String,
}

const SYNC_URL: &str = "https://api.todoist.com/sync/v9/sync";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let sync_url = args.sync_url.unwrap_or(SYNC_URL.into());

    let data_dir = if let Some(dir) = args.local_dir {
        PathBuf::from_str(dir.as_str()).unwrap()
    } else {
        let data_dir = dirs::data_local_dir().unwrap();
        data_dir.join("tuido")
    };

    let api_key = get_api_key(&data_dir)?;

    // FIXME: probably want to split up the network/file responsibilities here
    let stored_user = get_stored_user_data(&data_dir, &sync_url, &api_key).await?;

    if let Some(new_todo) = args.add_todo {
        let add_item_response = add_item(
            sync_url,
            api_key,
            stored_user.inbox_project_id,
            new_todo.clone(),
        )
        .await;

        if add_item_response.is_ok() {
            println!("Todo '{new_todo}' added to inbox.");
        }
    }

    println!("Bye!");
    Ok(())
}

fn get_api_key(data_dir: &PathBuf) -> Result<String, Box<dyn Error>> {
    let auth_file_name = "client_auth.toml";
    let auth_path = Path::new(data_dir).join(auth_file_name);
    let file = fs::read_to_string(auth_path)?;
    let config: Config = toml::from_str(file.as_str())?;
    Ok(config.api_key)
}

async fn get_stored_user_data(
    data_dir: &PathBuf,
    sync_url: &String,
    api_key: &String,
) -> Result<User, Box<dyn Error>> {
    let user_storage_path = Path::new(data_dir).join("data").join("user.json");

    if user_storage_path.exists() {
        let file = fs::read_to_string(user_storage_path)?;
        let user = serde_json::from_str::<User>(&file)?;
        Ok(user)
    } else {
        let user = get_user(sync_url, api_key).await?;
        // store in file
        println!("Storing user data in '{}'.", user_storage_path.display());
        fs::create_dir_all(Path::new(data_dir).join("data"))?;
        let file = fs::File::create(user_storage_path)?;
        serde_json::to_writer_pretty(file, &user)?;
        Ok(user)
    }
}

async fn add_item(
    sync_url: String,
    api_key: String,
    project_id: String,
    item: String,
) -> Result<Response, Box<dyn Error>> {
    let request_body = AddItemRequest {
        sync_token: "*".to_string(),
        resource_types: vec![],
        commands: vec![AddItemCommand {
            request_type: "item_add".to_string(),
            args: AddItemRequestArgs {
                project_id,
                content: item,
            },
            temp_id: Uuid::new_v4(),
            uuid: Uuid::new_v4(),
        }],
    };

    let client = reqwest::Client::new();
    let resp = match client
        .post(sync_url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&request_body)
        .send()
        .await
    {
        Ok(resp) => resp.json::<Response>().await?,
        Err(err) => panic!("Error: {err}"),
    };

    Ok(resp)
}

pub async fn get_user(sync_url: &String, api_key: &String) -> Result<User, Box<dyn Error>> {
    print!("Fetching user data... ");
    let request_body = GetUserRequest {
        sync_token: "*".to_string(),
        resource_types: vec!["user".to_string()],
        commands: vec![],
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(sync_url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&request_body)
        .send()
        .await
        .map(reqwest::Response::json::<Response>)?
        .await?;

    println!("done.");
    Ok(resp.user.unwrap())
}
