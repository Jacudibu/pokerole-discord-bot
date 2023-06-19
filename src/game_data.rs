use std::collections::HashMap;
use std::sync::Arc;
use sqlx::{Pool, Row, Sqlite};
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

    character_name_cache: Arc<Vec<String>>,
}

impl Data {
    pub async fn new(database: Pool<Sqlite>, game: Arc<GameData>) -> Self {
        let mut result = Data {
            database, game,
            character_name_cache: Arc::new(Vec::default())
        };

        result.update_character_name_cache().await;

        result
    }
}

impl Data {
    pub fn get_character_name_cache(&self) -> &Arc<Vec<String>> {
        &self.character_name_cache
    }

    pub async fn update_character_name_cache(&mut self) {
        let entries = sqlx::query("SELECT name FROM characters")
            .fetch_all(&self.database).await;

        if let Ok(entries) = entries {
            self.character_name_cache = Arc::new(entries.iter().map(|x| x.get::<String, usize>(0)).collect())
        }
    }


}
