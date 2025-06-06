use crate::shared::game_data::parser::file_reader;
use crate::shared::game_data::parser::issue_handler::IssueLogger;
use std::path::Path;

use crate::shared::game_data::pokerole_data::raw_ability::RawPokeroleAbility;
use crate::shared::game_data::pokerole_data::raw_item::RawPokeroleItem;
use crate::shared::game_data::pokerole_data::raw_move::RawPokeroleMove;
use crate::shared::game_data::pokerole_data::raw_nature::RawPokeroleNature;
use crate::shared::game_data::pokerole_data::raw_pokemon::RawPokerolePokemon;

pub struct PokeroleDataBundle {
    pub abilities: Vec<RawPokeroleAbility>,
    pub items: Vec<RawPokeroleItem>,
    pub moves: Vec<RawPokeroleMove>,
    pub natures: Vec<RawPokeroleNature>,
    pub pokemon: Vec<RawPokerolePokemon>,
}

pub fn parse(repo_path: &Path) -> PokeroleDataBundle {
    let logger = &mut IssueLogger;
    let mut items: Vec<RawPokeroleItem> =
        file_reader::parse_directory(repo_path, "Version20/Items", logger);
    items.extend(file_reader::parse_directory(
        repo_path,
        "Homebrew/Items",
        logger,
    ));

    PokeroleDataBundle {
        abilities: file_reader::parse_directory(repo_path, "Version20/Abilities", logger),
        items,
        moves: file_reader::parse_directory(repo_path, "Version20/Moves", logger),
        natures: file_reader::parse_directory(repo_path, "Version20/Natures", logger),
        pokemon: file_reader::parse_directory(repo_path, "Version20/Pokedex", logger),
    }
}
