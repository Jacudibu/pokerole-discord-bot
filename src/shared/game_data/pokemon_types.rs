use crate::shared::enums::PokemonType;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct PokemonTypes {
    pub type1: PokemonType,
    pub type2: Option<PokemonType>,
}

impl PokemonTypes {
    pub fn parse_types(type1: &str, type2: &str) -> Self {
        Self {
            type1: Self::parse_type(type1).unwrap(),
            type2: Self::parse_type(type2),
        }
    }

    fn parse_type(raw: &str) -> Option<PokemonType> {
        if raw.is_empty() {
            return None;
        }

        Some(PokemonType::from_str(raw).unwrap())
    }
}
