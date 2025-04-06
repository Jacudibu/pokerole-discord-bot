use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{pokemon_from_autocomplete_string, Error};
use crate::shared::{emoji, PoiseContext};

/// Display status effects
#[poise::command(slash_command)]
pub async fn efficiency(
    ctx: PoiseContext<'_>,
    #[description = "Get a typechart for a certain mon."]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
) -> Result<(), Error> {
    let pokemon = pokemon_from_autocomplete_string(&ctx, &name).await?;
    let emoji = emoji::get_any_pokemon_emoji_with_space(
        ctx.serenity_context(),
        &ctx.data().database,
        pokemon,
    )
    .await;
    ctx.say(
        &ctx.data()
            .game
            .type_efficiency
            .get_type_resistances_string(pokemon, emoji),
    )
    .await?;

    Ok(())
}
