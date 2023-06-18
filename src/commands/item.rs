use crate::commands::{Context, Error};
use crate::commands::autocompletion::autocomplete_item;

/// Display item description
#[poise::command(slash_command)]
pub async fn item(
    ctx: Context<'_>,
    #[description = "Which item?"]
    #[rename = "name"]
    #[autocomplete = "autocomplete_item"]
    name: String,
) -> Result<(), Error> {
    if let Some(item) = ctx.data().game.items.get(&name.to_lowercase()) {
        ctx.say(item.build_string()).await?;
    } else {
        ctx.send(|b| {
            b.content(std::format!("Unable to find an item named **{}**, sorry!", name));
            b.ephemeral(true)
        }).await?;
    }

    Ok(())
}
