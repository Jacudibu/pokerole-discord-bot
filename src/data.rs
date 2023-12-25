use crate::cache::Cache;
use crate::game_data::GameData;
use sqlx::{Pool, Sqlite};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct Data {
    pub database: Pool<Sqlite>,
    pub game: Arc<GameData>,
    pub cache: Arc<Cache>,
    pub is_backup_thread_running: AtomicBool,
    pub is_weekly_reset_thread_running: AtomicBool,
}

impl Data {
    pub async fn new(database: Pool<Sqlite>, game: Arc<GameData>) -> Self {
        let result = Data {
            database,
            game,
            cache: Arc::new(Cache::new()),
            is_backup_thread_running: AtomicBool::new(false),
            is_weekly_reset_thread_running: AtomicBool::new(false),
        };

        result.cache.update_character_names(&result.database).await;

        result
    }
}
