use crate::commands::{Context, Error};
use crate::commands::autocompletion::autocomplete_status_effect;

/// Display status effects
#[poise::command(slash_command)]
pub async fn status(
    ctx: Context<'_>,
    #[description = "Which status effect?"]
    #[rename = "name"]
    #[autocomplete = "autocomplete_status_effect"]
    name: String,
) -> Result<(), Error> {
    if let Some(status_effect) = ctx.data().game.status_effects.get(&name.to_lowercase()) {
        ctx.say(status_effect.build_string()).await?;
    } else {
        ctx.send(|b| {
            b.content(std::format!("Unable to find a status effect named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name));
            b.ephemeral(true)
        }).await?;
    }

    Ok(())
}
