#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use todoist_tui::{
    cli::{self, Args},
    storage::{
        config_manager::ConfigManager, file_manager::FileManager, model_manager::ModelManager,
    },
    sync::client::Client,
    tui,
};

#[derive(Serialize, Deserialize)]
struct Config {
    api_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let file_manager = FileManager::init(args.local_dir_override.as_deref())?;
    let config_manager = ConfigManager::new(&file_manager);
    let model_manager = ModelManager::new(&file_manager);

    let client = config_manager
        .get_api_token()
        .map(|token| Client::new(&token, args.sync_url_override.as_deref()));

    if let Some(ref command) = args.command.clone() {
        cli::handle_command(command, args, model_manager, client, config_manager).await?;
    } else {
        tui::run(model_manager, client).await?;
    }

    Ok(())
}
