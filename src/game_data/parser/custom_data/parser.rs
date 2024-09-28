use crate::game_data::parser::custom_data::custom_ability::CustomAbility;
use crate::game_data::parser::custom_data::custom_item::CustomItem;
use crate::game_data::parser::custom_data::custom_move::CustomMove;
use crate::game_data::parser::custom_data::custom_pokemon::CustomPokemon;
use crate::game_data::parser::custom_data::custom_potion::CustomPotion;
use crate::game_data::parser::custom_data::custom_status_effect::CustomStatusEffect;
use crate::game_data::parser::custom_data::custom_weather::CustomWeather;
use crate::game_data::parser::file_reader;
use crate::game_data::parser::issue_handler::IssueStorage;

pub struct CustomDataBundle {
    pub abilities: Vec<CustomAbility>,
    pub pokemon: Vec<CustomPokemon>,
    pub moves: Vec<CustomMove>,
    pub items: Vec<CustomItem>,
    pub status_effects: Vec<CustomStatusEffect>,
    pub potions: Vec<CustomPotion>,
    pub weather: Vec<CustomWeather>,
}

pub fn parse(custom_data_path: &str) -> (CustomDataBundle, IssueStorage) {
    let mut issues = IssueStorage::default();
    (
        CustomDataBundle {
            abilities: file_reader::parse_directory(
                custom_data_path.to_owned() + "Abilities",
                &mut issues,
            ),
            pokemon: file_reader::parse_directory(
                custom_data_path.to_owned() + "Pokedex",
                &mut issues,
            ),
            moves: file_reader::parse_directory(custom_data_path.to_owned() + "Moves", &mut issues),
            items: file_reader::parse_directory(custom_data_path.to_owned() + "Items", &mut issues),
            status_effects: file_reader::parse_directory(
                custom_data_path.to_owned() + "StatusEffects",
                &mut issues,
            ),
            potions: file_reader::parse_directory(
                custom_data_path.to_owned() + "Potions",
                &mut issues,
            ),
            weather: file_reader::parse_directory(
                custom_data_path.to_owned() + "Weather",
                &mut issues,
            ),
        },
        issues,
    )
}
