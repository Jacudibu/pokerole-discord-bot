//! Contains stuff that should probably be further refactored somehow at some point

use crate::shared::game_data::pokemon::Pokemon;
use crate::shared::helpers;
use poise::CreateReply;
use serenity::all::CreateActionRow;

pub fn create_learns_reply(pokemon: &Pokemon, emoji: String) -> CreateReply {
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
