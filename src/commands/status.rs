use crate::commands::autocompletion::autocomplete_status_effect;
use crate::commands::Error;
use crate::shared::PoiseContext;
use poise::CreateReply;

/// Display status effects
#[poise::command(slash_command)]
pub async fn status(
    ctx: PoiseContext<'_>,
    #[description = "Which status effect?"]
    #[rename = "name"]
    #[autocomplete = "autocomplete_status_effect"]
    name: String,
) -> Result<(), Error> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;
    if let Some(status_effect) = game_data.status_effects.get(&name.to_lowercase()) {
        ctx.say(status_effect.build_string()).await?;
    } else {
        ctx.send(CreateReply::default()
            .content(std::format!("Unable to find a status effect named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name))
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}
