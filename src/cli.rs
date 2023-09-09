use crate::{
    model::{item::Item, Model},
    sync::{client::Client, Request, ResourceType},
};
use anyhow::{anyhow, Result};
use std::io::{self, Write};

/// # Errors
///
/// Returns an error if `number` does not correspond to a valid item
pub fn complete_item(number: usize, model: &mut Model) -> Result<&Item> {
    // look at the current inbox and determine which task is targeted
    let inbox_items = model.get_inbox_items();
    let num_items = inbox_items.len();

    let error_msg = || {
        anyhow!(
            "'{number}' is outside of the valid range. Pass a number between 1 and {num_items}.",
        )
    };

    let item = inbox_items.get(number - 1).ok_or_else(error_msg)?;

    // update the item's status
    let completed_item = model
        .complete_item(&item.id.clone())
        .map_err(|_| error_msg())?;

    Ok(completed_item)
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
