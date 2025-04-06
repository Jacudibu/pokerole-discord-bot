pub mod api_types;
pub mod pokemon_api_parser;

#[derive(Debug, serde::Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub struct PokemonApiId(pub u16);
