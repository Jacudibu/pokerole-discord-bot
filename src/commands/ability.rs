use crate::commands::autocompletion::autocomplete_ability;
use crate::commands::{Context, Error};
use poise::CreateReply;

/// Display an Ability
#[poise::command(slash_command)]
pub async fn ability(
    ctx: Context<'_>,
    #[description = "Which ability?"]
    #[rename = "ability"]
    #[autocomplete = "autocomplete_ability"]
    name: String,
) -> Result<(), Error> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;

    if let Some(ability) = game_data.abilities.get(&name.to_lowercase()) {
        ctx.say(ability.build_string("")).await?;
    } else {
        ctx.send(CreateReply::default()
            .content(std::format!("Unable to find an ability named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name))
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}
