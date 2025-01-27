use crate::commands::{Context, Error};
use crate::game_data::GameData;
use rand::prelude::IteratorRandom;

/// Use the most randomest of moves!
#[poise::command(slash_command)]
pub async fn metronome(ctx: Context<'_>) -> Result<(), Error> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;

    ctx.say(get_metronome_text(&game_data)).await?;
    Ok(())
}

pub fn get_metronome_text(data: &GameData) -> String {
    let move_name = data
        .move_names
        .iter()
        .choose(&mut rand::rng())
        .expect("There should be a name.");
    if let Some(poke_move) = data.moves.get(&move_name.to_lowercase()) {
        poke_move.build_string()
    } else {
        format!("Error: randomness rolled {}, but there was no move with that name defined? This should never happen. D:", move_name)
    }
}
