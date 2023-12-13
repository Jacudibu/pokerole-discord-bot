use crate::commands::autocompletion::autocomplete_item;
use crate::commands::{Context, Error};
use poise::CreateReply;

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
        ctx.send(
            CreateReply::default()
                .content(std::format!(
                    "Unable to find an item named **{}**, sorry!",
                    name
                ))
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}
