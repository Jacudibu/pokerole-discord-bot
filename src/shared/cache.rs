use log::error;
use sqlx::{Pool, Sqlite};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct CharacterCacheItem {
    pub id: i64,
    pub name: String,
    pub is_retired: bool,
    pub guild_id: u64,
    pub user_id: u64,
    autocomplete_name: String,
}

impl CharacterCacheItem {
    pub fn new(
        id: i64,
        name: String,
        user_id: u64,
        guild_id: u64,
        is_retired: bool,
        user_nickname: String,
    ) -> Self {
        CharacterCacheItem {
            autocomplete_name: CharacterCacheItem::build_autocomplete_name(&name, &user_nickname),
            id,
            user_id,
            guild_id,
            is_retired,
            name,
        }
    }

    pub fn get_autocomplete_name(&self) -> &String {
        &self.autocomplete_name
    }

    fn build_autocomplete_name(name: &str, nickname: &str) -> String {
        format!("{} (@{})", name, nickname)
    }
}

#[derive(Debug, Clone)]
pub struct WalletCacheItem {
    pub id: i64,
    pub name: String,
    pub guild_id: u64,
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

    pub async fn get_character(&self, id: i64) -> Option<CharacterCacheItem> {
        self.character_cache
            .lock()
            .await
            .iter()
            .find(|x| x.id == id)
            .cloned()
    }

    pub async fn reset(&self, db: &Pool<Sqlite>) {
        self.update_character_names(db).await;
    }

    pub async fn update_character_names(&self, db: &Pool<Sqlite>) {
        let entries = sqlx::query!(
"SELECT character.id, character.name as character_name, character.user_id, character.guild_id, character.is_retired, user_in_guild.name as user_name
FROM character
LEFT JOIN user_in_guild ON
    user_in_guild.user_id = character.user_id AND
    user_in_guild.guild_id = character.guild_id
")
            .fetch_all(db).await;

        match entries {
            Ok(entries) => {
                let mut cache = self.character_cache.lock().await;
                cache.clear();
                for x in entries {
                    cache.push(CharacterCacheItem::new(
                        x.id,
                        x.character_name,
                        x.user_id as u64,
                        x.guild_id as u64,
                        x.is_retired,
                        x.user_name.unwrap_or(String::new()),
                    ))
                }
            }
            Err(e) => {
                error!("Was unable to update character names! {}", e);
            }
        }
    }
}
