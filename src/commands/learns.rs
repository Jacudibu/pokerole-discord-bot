use poise::CreateReply;
use crate::commands::{Context, Error};
use crate::commands::autocompletion::autocomplete_pokemon;
use crate::game_data::pokemon::Pokemon;
use crate::helpers;

/// Display Pokemon moves
#[poise::command(slash_command, prefix_command)]
pub async fn learns(
    ctx: Context<'_>,
    #[description = "Which pokemon?"]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
) -> Result<(), Error> {
    if let Some(pokemon) = ctx.data().game.pokemon.get(&name.to_lowercase()) {
        ctx.send(|b| create_reply(b, pokemon)).await?;
    } else {
        ctx.send(|b| {
            b.content(std::format!("Unable to find a pokemon named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name));
            b.ephemeral(true)
        }).await?;
    }

    Ok(())
}

pub fn create_reply<'a, 'b>(b: &'a mut CreateReply<'b>, pokemon: &Pokemon) -> &'a mut CreateReply<'b> {
    b.content(pokemon.build_move_string())
        .components(|b| {
            b.create_action_row(|b| {
                b.add_button(helpers::create_button("Show All Learnable Moves", format!("learns-all_{}", pokemon.name.to_lowercase()).as_str()))
            })
        })
}
