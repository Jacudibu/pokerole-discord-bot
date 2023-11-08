use crate::game_data::pokemon_api::PokemonApiId;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) mod ability;
pub(crate) mod item;
pub(crate) mod r#move;
pub(crate) mod nature;
pub(crate) mod pokemon;
pub(crate) mod potion;
pub(crate) mod rule;
pub(crate) mod status_effect;
pub(crate) mod weather;

mod pokemon_api;
mod pokerole_data;
mod pokerole_discord_py_data;

pub(in crate::game_data) mod enums;

pub mod parser;
pub(crate) mod type_efficiency;

/// Data which is stored and accessible in all command invocations
pub struct GameData {
    pub abilities: Arc<HashMap<String, ability::Ability>>,
    pub ability_names: Arc<Vec<String>>,
    pub potions: Arc<HashMap<String, potion::Potion>>,
    pub potion_names: Arc<Vec<String>>,
    pub items: Arc<HashMap<String, item::Item>>,
    pub item_names: Arc<Vec<String>>,
    pub moves: Arc<HashMap<String, r#move::Move>>,
    pub move_names: Arc<Vec<String>>,
    pub natures: Arc<HashMap<String, nature::Nature>>,
    pub nature_names: Arc<Vec<String>>,
    pub pokemon: Arc<HashMap<String, pokemon::Pokemon>>,
    pub pokemon_by_api_id: Arc<HashMap<PokemonApiId, pokemon::Pokemon>>,
    pub pokemon_names: Arc<Vec<String>>,
    pub status_effects: Arc<HashMap<String, status_effect::StatusEffect>>,
    pub status_effects_names: Arc<Vec<String>>,
    pub weather: Arc<HashMap<String, weather::Weather>>,
    pub weather_names: Arc<Vec<String>>,
    pub rule_names: Arc<Vec<String>>,
    pub rules: Arc<HashMap<String, rule::Rule>>,
    pub type_efficiency: Arc<type_efficiency::TypeEfficiency>,
}
