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
    if let Some(ability) = ctx.data().game.abilities.get(&name.to_lowercase()) {
        ctx.say(ability.build_string("")).await?;
    } else {
        ctx.send(CreateReply::default()
            .content(std::format!("Unable to find an ability named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name))
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}
