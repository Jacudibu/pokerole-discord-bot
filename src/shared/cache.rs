use serenity::all::{GuildId, GuildInfo, UserId};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use tokio::sync::{Mutex, MutexGuard};

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

#[derive(Default)]
pub struct Cache {
    guild_cache: Mutex<HashMap<GuildId, GuildCacheItem>>,
    character_cache: Mutex<HashMap<i64, CharacterCacheItem>>,
}

pub struct GuildCacheItem {
    member_names: HashMap<UserId, String>,
}

fn get_user_name(
    guild_cache: &HashMap<GuildId, GuildCacheItem>,
    guild_id: &GuildId,
    user_id: &UserId,
) -> String {
    let Some(guild) = guild_cache.get(guild_id) else {
        return "??? No Guild".into();
    };

    if let Some(name) = guild.member_names.get(user_id) {
        name.clone()
    } else {
        "??? No User".into()
    }
}

impl Cache {
    pub async fn rebuild_everything(&self, context: &serenity::all::Context, db: &Pool<Sqlite>) {
        let Ok(guilds) = context.http.get_guilds(None, None).await else {
            todo!()
        };

        let mut guild_cache = self.guild_cache.lock().await;
        let mut character_cache = self.character_cache.lock().await;

        Self::rebuild_guild_cache(&mut guild_cache, context, guilds).await;
        Self::rebuild_character_cache(&mut character_cache, db, &guild_cache).await;
    }

    pub async fn update_character_names(&self, db: &Pool<Sqlite>) {
        let guild_cache = self.guild_cache.lock().await;
        let mut character_cache = self.character_cache.lock().await;

        Self::rebuild_character_cache(&mut character_cache, db, &guild_cache).await;
    }

    pub async fn update_or_add_user_name(
        &self,
        guild_id: &GuildId,
        user_id: &UserId,
        new_name: String,
        db: &Pool<Sqlite>,
    ) {
        let mut guild_cache = self.guild_cache.lock().await;

        let Some(guild) = guild_cache.get_mut(guild_id) else {
            todo!()
        };

        guild.member_names.insert(*user_id, new_name);

        // TODO: Big performance gains if we only update the characters this user owns... :')
        let mut character_cache = self.character_cache.lock().await;
        Self::rebuild_character_cache(&mut character_cache, db, &guild_cache).await;
    }

    pub async fn get_characters(&self) -> MutexGuard<'_, HashMap<i64, CharacterCacheItem>> {
        self.character_cache.lock().await
    }

    pub async fn get_character(&self, id: i64) -> Option<CharacterCacheItem> {
        self.character_cache.lock().await.get(&id).cloned()
    }

    async fn rebuild_guild_cache(
        guild_cache: &mut HashMap<GuildId, GuildCacheItem>,
        context: &serenity::all::Context,
        guilds: Vec<GuildInfo>,
    ) {
        guild_cache.clear();

        for x in guilds {
            let Ok(members) = context.http.get_guild_members(x.id, None, None).await else {
                todo!()
            };

            let mut member_names = HashMap::new();
            for member in members {
                if let Some(nickname) = member.nick {
                    member_names.insert(member.user.id, nickname);
                } else {
                    member_names.insert(member.user.id, member.user.name);
                }
            }

            guild_cache.insert(x.id, GuildCacheItem { member_names });
        }
    }

    async fn rebuild_character_cache(
        character_cache: &mut HashMap<i64, CharacterCacheItem>,
        db: &Pool<Sqlite>,
        guild_cache: &HashMap<GuildId, GuildCacheItem>,
    ) {
        character_cache.clear();

        let Ok(entries) =
            sqlx::query!("SELECT id, name, user_id, guild_id, is_retired FROM character")
                .fetch_all(db)
                .await
        else {
            todo!()
        };

        for record in entries {
            let guild_id = GuildId::from(record.guild_id as u64);
            let user_id = UserId::from(record.user_id as u64);
            let user_name = get_user_name(guild_cache, &guild_id, &user_id);

            character_cache.insert(
                record.id,
                CharacterCacheItem::new(
                    record.id,
                    record.name,
                    record.user_id as u64,
                    record.guild_id as u64,
                    record.is_retired,
                    user_name,
                ),
            );
        }
    }
}
