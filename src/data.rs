use std::sync::Arc;
use sqlx::{Pool, Sqlite};
use crate::cache::Cache;
use crate::game_data::GameData;

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
