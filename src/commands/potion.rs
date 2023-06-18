use crate::commands::{Context, Error};
use crate::commands::autocompletion::autocomplete_potion;

/// List potion effects and crafting recipes
#[poise::command(slash_command)]
pub async fn potion(
    ctx: Context<'_>,
    #[description = "Which item?"]
    #[rename = "name"]
    #[autocomplete = "autocomplete_potion"]
    name: String,
) -> Result<(), Error> {
    if let Some(potion) = ctx.data().game.potions.get(&name.to_lowercase()) {
        ctx.say(potion.build_string()).await?;
    } else {
        ctx.send(|b| {
            b.content(std::format!("Unable to find a potion named **{}**, sorry!", name));
            b.ephemeral(true)
        }).await?;
    }

    Ok(())
}
