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
    if let Some(nature) = ctx.data().game.natures.get(&name.to_lowercase()) {
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
