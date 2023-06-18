use std::collections::HashMap;
use std::sync::Arc;
use sqlx::{Pool, Sqlite};
use crate::data::ability::Ability;
use crate::data::rule::Rule;
use crate::data::item::Item;
use crate::data::nature::Nature;
use crate::data::pokemon::Pokemon;
use crate::data::potion::Potion;
use crate::data::r#move::Move;
use crate::data::status_effect::StatusEffect;
use crate::data::type_efficiency::TypeEfficiency;
use crate::data::weather::Weather;

/// Data which is stored and accessible in all command invocations
pub struct GameData {
    pub abilities: Arc<HashMap<String, Ability>>,
    pub ability_names: Arc<Vec<String>>,
    pub potions: Arc<HashMap<String, Potion>>,
    pub potion_names: Arc<Vec<String>>,
    pub items: Arc<HashMap<String, Item>>,
    pub item_names: Arc<Vec<String>>,
    pub moves: Arc<HashMap<String, Move>>,
    pub move_names: Arc<Vec<String>>,
    pub natures: Arc<HashMap<String, Nature>>,
    pub nature_names: Arc<Vec<String>>,
    pub pokemon: Arc<HashMap<String, Pokemon>>,
    pub pokemon_names: Arc<Vec<String>>,
    pub status_effects: Arc<HashMap<String, StatusEffect>>,
    pub status_effects_names: Arc<Vec<String>>,
    pub weather: Arc<HashMap<String, Weather>>,
    pub weather_names: Arc<Vec<String>>,
    pub rule_names: Arc<Vec<String>>,
    pub rules: Arc<HashMap<String, Rule>>,
    pub type_efficiency: Arc<TypeEfficiency>,
}

pub struct Data {
    pub database: Pool<Sqlite>,
    pub game: Arc<GameData>,
}
