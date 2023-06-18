use crate::commands::{Context, Error};
use rand::seq::SliceRandom;
use crate::game_data::GameData;

/// Use the most randomest of moves!
#[poise::command(slash_command)]
pub async fn metronome(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.say(get_metronome_text(&ctx.data().game)).await?;
    Ok(())
}

pub fn get_metronome_text(data: &GameData) -> String {
    let move_name = data.move_names.choose(&mut rand::thread_rng()).expect("There should be a name.");
    return if let Some(poke_move) = data.moves.get(&move_name.to_lowercase()) {
        poke_move.build_string()
    } else {
        format!("Error: randomness rolled {}, but there was no move with that name defined? This should never happen. D:", move_name)
    }
}
