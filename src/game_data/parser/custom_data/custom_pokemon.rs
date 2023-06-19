use serde::Deserialize;
use crate::enums::RegionalVariant;

#[derive(Debug, Deserialize)]
pub struct CustomPokemon {
    pub number: u16,
    pub variant: Option<RegionalVariant>,
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
    pub bronze: Vec<String>,
    pub silver: Vec<String>,
    pub gold: Vec<String>,
    pub platinum: Vec<String>,
    pub diamond: Vec<String>,
}
