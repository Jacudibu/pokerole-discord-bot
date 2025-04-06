use crate::shared::enums::RegionalVariant;
use crate::shared::game_data::pokemon_api::PokemonApiId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CustomPokemon {
    pub number: u16,
    pub api_id: Option<PokemonApiId>,
    pub variant: Option<RegionalVariant>,
    pub evolves_from_override: Option<PokemonApiId>,
    pub name: String,
    pub base_hp: u8,
    pub strength: String,
    pub dexterity: String,
    pub vitality: String,
    pub special: String,
    pub insight: String,
    pub moves: CustomPokemonMoves,
}

#[derive(Debug, Deserialize)]
pub struct CustomPokemonMoves {
    pub bronze: Option<Vec<String>>,
    pub silver: Option<Vec<String>>,
    pub gold: Option<Vec<String>>,
    pub platinum: Option<Vec<String>>,
    pub diamond: Option<Vec<String>>,
}
