use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{pokemon_from_autocomplete_string, Context, Error};
use crate::{emoji, helpers};
use poise::CreateReply;
use serenity::all::CreateActionRow;
use std::default::Default;

async fn print_poke_stats(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let pokemon = pokemon_from_autocomplete_string(&ctx, &name)?;
    let emoji = emoji::get_any_pokemon_emoji_with_space(&ctx.data().database, pokemon).await;
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
    ctx: Context<'_>,
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
    ctx: Context<'_>,
    #[description = "Which pokemon?"]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
) -> Result<(), Error> {
    print_poke_stats(ctx, name).await
}

fn create_buttons<'a>(name: &String) -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        helpers::create_button("Abilities", format!("abilities_{}", name).as_str(), false),
        helpers::create_button(
            "Type Effectiveness",
            format!("efficiency_{}", name).as_str(),
            false,
        ),
        helpers::create_button("Moves", format!("moves_{}", name).as_str(), false),
    ])
}
