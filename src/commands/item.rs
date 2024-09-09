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
    let game_data = ctx.data().game_multi_source.get_by_context(&ctx).await;

    if let Some(item) = game_data.items.get(&name.to_lowercase()) {
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
