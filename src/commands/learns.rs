use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{pokemon_from_autocomplete_string, Context, Error};
use crate::game_data::pokemon::Pokemon;
use crate::{emoji, helpers};
use poise::CreateReply;
use serenity::all::CreateActionRow;

/// Display Pokemon moves
#[poise::command(slash_command, prefix_command)]
pub async fn learns(
    ctx: Context<'_>,
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

    ctx.send(create_reply(pokemon, emoji)).await?;

    Ok(())
}

pub fn create_reply(pokemon: &Pokemon, emoji: String) -> CreateReply {
    CreateReply::default()
        .content(pokemon.build_move_string(emoji))
        .components(vec![CreateActionRow::Buttons(vec![
            helpers::create_button(
                "Show All Learnable Moves",
                format!("learns-all_{}", pokemon.name.to_lowercase()).as_str(),
                false,
            ),
        ])])
}
