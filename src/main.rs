use std::{collections::HashMap, fs, path::Path};

use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    api_key: String,
}

#[tokio::main]
async fn main() {
    let home_dir = dirs::home_dir().unwrap();
    let config_segment = ".config";
    let app_segment = "tuido";
    let auth_file = "client_auth.toml";

    let auth_path = Path::new(&home_dir)
        .join(config_segment)
        .join(app_segment)
        .join(auth_file);

    let file = fs::read_to_string(&auth_path).unwrap();
    let config: Config = toml::from_str(file.as_str()).unwrap();

    println!("home: {}", auth_path.to_str().unwrap());
    println!("api_key: {}", config.api_key);

    // get_projects().await;
}

pub async fn get_projects() {
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
