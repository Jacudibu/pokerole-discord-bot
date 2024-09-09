use crate::game_data::GameData;
use std::collections::HashMap;

pub fn parse(base_path: String, base_data: &GameData) -> HashMap<i64, GameData> {
    // TODO: Load array with custom data folders

    // TODO: loop through that array, clone & override data as needed
    HashMap::default()
}

pub fn parse_custom(base_data: GameData) -> GameData {
    let data = base_data.clone();

    data
}
