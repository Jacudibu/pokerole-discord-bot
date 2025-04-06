use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{pokemon_from_autocomplete_string, Error};
use crate::shared::{clunky_stuff, emoji, PoiseContext};
/// Display Pokemon moves
#[poise::command(slash_command, prefix_command)]
pub async fn learns(
    ctx: PoiseContext<'_>,
    #[description = "Which pokemon?"]
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

    ctx.send(clunky_stuff::create_learns_reply(pokemon, emoji))
        .await?;

    Ok(())
}
