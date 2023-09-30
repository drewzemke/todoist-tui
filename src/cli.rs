use crate::{
    model::Model,
    sync::{client::Client, Request, ResourceType},
};
use anyhow::{anyhow, Result};
use std::io::{self, Write};

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
        resource_types: vec![ResourceType::All],
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
