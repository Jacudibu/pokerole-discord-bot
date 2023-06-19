use std::collections::HashMap;
use std::sync::{Arc};
use log::{error};
use sqlx::{Pool, Row, Sqlite};
use tokio::sync::Mutex;
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

pub struct Cache {
    character_names: Mutex<Vec<String>>,
}

impl Cache {
    fn new() -> Cache {
        Cache {
            character_names: Mutex::new(Vec::new()),
        }
    }

    pub async fn get_character_names(&self) -> Vec<String> {
        self.character_names.lock().await.clone()
    }

    pub async fn update_character_names(&self, db: &Pool<Sqlite>) {
        let entries = sqlx::query("SELECT name FROM characters")
            .fetch_all(db).await;

        if let Ok(entries) = entries {
            let mut names = self.character_names.lock().await;
            names.clear();
            names.extend(entries.iter().map(|x| x.get::<String, usize>(0)));
        } else {
            error!("Was unable to update character names!");
        }
    }
}

pub struct Data {
    pub database: Pool<Sqlite>,
    pub game: Arc<GameData>,
    pub cache: Arc<Cache>,
}

impl Data {
    pub async fn new(database: Pool<Sqlite>, game: Arc<GameData>) -> Self {
        let result = Data {
            database, game,
            cache: Arc::new(Cache::new()),
        };

        result.cache.update_character_names(&result.database).await;

        result
    }
}

impl Cache {



}
