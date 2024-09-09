use crate::commands::autocompletion::autocomplete_nature;
use crate::commands::{Context, Error};
use poise::CreateReply;

/// Display an Ability
#[poise::command(slash_command)]
pub async fn nature(
    ctx: Context<'_>,
    #[description = "Which nature?"]
    #[rename = "nature"]
    #[autocomplete = "autocomplete_nature"]
    name: String,
) -> Result<(), Error> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;

    if let Some(nature) = game_data.natures.get(&name.to_lowercase()) {
        ctx.say(nature.build_string()).await?;
    } else {
        ctx.send(
            CreateReply::default()
                .content(std::format!(
                    "Unable to find a nature named **{}**, sorry!",
                    name
                ))
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}
