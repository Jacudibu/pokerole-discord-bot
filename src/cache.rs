use tokio::sync::Mutex;
use sqlx::{Pool, Row, Sqlite};
use log::error;

#[derive(Debug, Clone)]
pub struct CharacterCacheItem {
    pub name: String,
    pub user_id: u64,
    pub guild_id: u64,
    autocomplete_name: String,
}

impl CharacterCacheItem {
    pub fn new(name: String, user_id: u64, guild_id: u64) -> Self {
        CharacterCacheItem {
            user_id,
            guild_id,
            name: name.clone(),
            autocomplete_name: name,
        }
    }

    pub fn get_autocomplete_name(&self) -> &String {
        &self.autocomplete_name
    }
}

pub struct Cache {
    character_cache: Mutex<Vec<CharacterCacheItem>>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            character_cache: Mutex::new(Vec::new()),
        }
    }

    pub async fn get_characters(&self) -> Vec<CharacterCacheItem> {
        self.character_cache.lock().await.clone()
    }

    pub async fn update_character_names(&self, db: &Pool<Sqlite>) {
        let entries = sqlx::query("SELECT name, user_id, guild_id FROM character")
            .fetch_all(db).await;

        if let Ok(entries) = entries {
            let mut cache = self.character_cache.lock().await;
            cache.clear();
            for x in entries {
                cache.push(CharacterCacheItem::new(
                    x.get::<String, usize>(0),
                    x.get::<i64, usize>(1) as u64,
                    x.get::<i64, usize>(2) as u64,
                ))
            }
        } else {
            error!("Was unable to update character names!");
        }
    }
}
