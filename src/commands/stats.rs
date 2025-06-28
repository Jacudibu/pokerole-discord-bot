use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{pokemon_from_autocomplete_string, Error};
use crate::shared::utility::button_building;
use crate::shared::{emoji, PoiseContext};
use poise::CreateReply;
use serenity::all::CreateActionRow;
use std::default::Default;

async fn print_poke_stats(ctx: PoiseContext<'_>, name: String) -> Result<(), Error> {
    let pokemon = pokemon_from_autocomplete_string(&ctx, &name).await?;
    let emoji = emoji::get_any_pokemon_emoji_with_space(
        ctx.serenity_context(),
        &ctx.data().database,
        pokemon,
    )
    .await;
    ctx.send(
        CreateReply::default()
            .content(pokemon.build_stats_string(emoji))
            .components(vec![create_buttons(&pokemon.name.to_lowercase())]),
    )
    .await?;

    Ok(())
}

/// Display Pokemon stats. Same as /stats.
#[poise::command(slash_command)]
pub async fn pokemon(
    ctx: PoiseContext<'_>,
    #[description = "Which pokemon?"]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
) -> Result<(), Error> {
    print_poke_stats(ctx, name).await
}

/// Display Pokemon stats. Same as /pokemon
#[poise::command(slash_command)]
pub async fn stats(
    ctx: PoiseContext<'_>,
    #[description = "Which pokemon?"]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
) -> Result<(), Error> {
    print_poke_stats(ctx, name).await
}

fn create_buttons<'a>(name: &String) -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        button_building::create_button("Abilities", format!("abilities_{}", name).as_str(), false),
        button_building::create_button(
            "Type Effectiveness",
            format!("efficiency_{}", name).as_str(),
            false,
        ),
        button_building::create_button("Moves", format!("moves_{}", name).as_str(), false),
    ])
}
