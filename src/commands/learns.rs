use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{Context, Error};
use crate::game_data::pokemon::Pokemon;
use crate::helpers;
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
    if let Some(pokemon) = ctx.data().game.pokemon.get(&name.to_lowercase()) {
        ctx.send(create_reply(pokemon)).await?;
    } else {
        ctx.send(CreateReply::default()
            .content(std::format!("Unable to find a pokemon named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name))
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}

pub fn create_reply(pokemon: &Pokemon) -> CreateReply {
    CreateReply::default()
        .content(pokemon.build_move_string())
        .components(vec![CreateActionRow::Buttons(vec![
            helpers::create_button(
                "Show All Learnable Moves",
                format!("learns-all_{}", pokemon.name.to_lowercase()).as_str(),
                false,
            ),
        ])])
}
