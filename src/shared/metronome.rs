use crate::shared::game_data::GameData;
use rand::prelude::IteratorRandom;

/// Returns the move string of a randomly selected metronome move.
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
