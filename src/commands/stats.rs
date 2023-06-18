use std::default::Default;
use serenity::builder::{CreateComponents};
use crate::commands::{Context, Error};
use crate::commands::autocompletion::autocomplete_pokemon;
use crate::helpers;

/// Display Pokemon stats
#[poise::command(slash_command)]
pub async fn stats(
    ctx: Context<'_>,
    #[description = "Which pokemon?"]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
) -> Result<(), Error> {
    if let Some(pokemon) = ctx.data().game.pokemon.get(&name.to_lowercase()) {
        ctx.send(|b| {
            b.content(pokemon.build_stats_string());
            b.components(|b| create_buttons(b, &pokemon.name.to_lowercase()))
        }).await?;
    } else {
        ctx.send(|b| {
            b.content(std::format!("Unable to find a pokemon named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name));
            b.ephemeral(true)
        }).await?;
    }

    Ok(())
}

fn create_buttons<'a>(b: &'a mut CreateComponents, name: &String) -> &'a mut CreateComponents {
    b.create_action_row(|b| {
        b.add_button(helpers::create_button("Abilities", format!("abilities_{}", name).as_str()));
        b.add_button(helpers::create_button("Type Effectiveness", format!("efficiency_{}", name).as_str()));
        b.add_button(helpers::create_button("Moves", format!("moves_{}", name).as_str()))
    })
}
