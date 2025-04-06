use crate::shared::game_data::parser::custom_data::custom_ability::CustomAbility;
use crate::shared::game_data::parser::custom_data::custom_item::CustomItem;
use crate::shared::game_data::parser::custom_data::custom_move::CustomMove;
use crate::shared::game_data::parser::custom_data::custom_pokemon::CustomPokemon;
use crate::shared::game_data::parser::custom_data::custom_potion::CustomPotion;
use crate::shared::game_data::parser::custom_data::custom_status_effect::CustomStatusEffect;
use crate::shared::game_data::parser::custom_data::custom_weather::CustomWeather;
use crate::shared::game_data::parser::file_reader;
use crate::shared::game_data::parser::issue_handler::IssueStorage;
use std::path::Path;

pub struct CustomDataBundle {
    pub abilities: Vec<CustomAbility>,
    pub pokemon: Vec<CustomPokemon>,
    pub moves: Vec<CustomMove>,
    pub items: Vec<CustomItem>,
    pub status_effects: Vec<CustomStatusEffect>,
    pub potions: Vec<CustomPotion>,
    pub weather: Vec<CustomWeather>,
}

pub fn parse(custom_data_path: &Path) -> (CustomDataBundle, IssueStorage) {
    let mut issues = IssueStorage::default();
    (
        CustomDataBundle {
            abilities: file_reader::parse_directory(custom_data_path, "Abilities", &mut issues),
            pokemon: file_reader::parse_directory(custom_data_path, "Pokedex", &mut issues),
            moves: file_reader::parse_directory(custom_data_path, "Moves", &mut issues),
            items: file_reader::parse_directory(custom_data_path, "Items", &mut issues),
            status_effects: file_reader::parse_directory(
                custom_data_path,
                "StatusEffects",
                &mut issues,
            ),
            potions: file_reader::parse_directory(custom_data_path, "Potions", &mut issues),
            weather: file_reader::parse_directory(custom_data_path, "Weather", &mut issues),
        },
        issues,
    )
}
