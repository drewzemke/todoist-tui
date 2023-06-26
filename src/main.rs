#![allow(unused)]
use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use serde::Deserialize;

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

    let data_dir = match args.local_dir {
        Some(dir) => PathBuf::from_str(dir.as_str()).unwrap(),
        None => {
            let data_dir = dirs::data_local_dir().unwrap();
            data_dir.join("tuido")
        }
    };

    let api_key = get_api_key(data_dir)?;
    println!("api key: {}", api_key);

    let user = get_user(sync_url, api_key).await?;
    println!("inbox id: {}", user.inbox_project_id);

    Ok(())
}

fn get_api_key(data_dir: PathBuf) -> Result<String, Box<dyn Error>> {
    let auth_file_name = "client_auth.toml";

    let auth_path = Path::new(&data_dir).join(auth_file_name);

    let file = fs::read_to_string(auth_path)?;
    let config: Config = toml::from_str(file.as_str())?;

    Ok(config.api_key)
}

#[derive(Debug, Deserialize)]
pub struct SyncResponse {
    user: User,
}

#[derive(Debug, Deserialize)]
pub struct User {
    full_name: String,
    inbox_project_id: String,
}

pub async fn get_user(sync_url: String, api_key: String) -> Result<User, Box<dyn Error>> {
    let mut map = HashMap::new();
    map.insert("sync_token", "*");
    map.insert("resource_types", "[\"user\"]");

    let client = reqwest::Client::new();
    let resp = match client
        .post(sync_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&map)
        .send()
        .await
    {
        Ok(resp) => resp.json::<SyncResponse>().await?,
        Err(err) => panic!("Error: {}", err),
    };

    Ok(resp.user)
}

pub async fn get_projects(api_key: String) {
    let sync_url = "https://api.todoist.com/sync/v9/sync";

    let mut map = HashMap::new();
    map.insert("sync_token", "*");
    map.insert("resource_types", "[\"projects\"]");

    let client = reqwest::Client::new();
    let resp = match client
        .post(sync_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&map)
        .send()
        .await
    {
        Ok(resp) => resp.text().await.unwrap(),
        Err(err) => panic!("Error: {}", err),
    };

    println!("{}", resp);
}
