use crate::Error;
use crate::shared::data::Data;
use crate::shared::game_data::parser::issue_handler::IssueStorage;
pub use crate::shared::game_data::pokemon_api::PokemonApiId;
use serenity::all::{ComponentInteraction, GuildId, UserId};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;

pub mod ability;
pub mod item;
pub mod r#move;
pub mod nature;
pub mod pokemon;
pub mod potion;
pub mod status_effect;
pub mod weather;

mod pokemon_api;
mod pokerole_data;

pub mod enums;

pub mod parser;
pub mod type_efficiency;

/// Data which is stored and accessible in all command invocations
#[derive(Clone)]
pub struct GameData {
    pub name: String,
    pub id: i64,
    pub abilities: HashMap<String, ability::Ability>,
    pub ability_names: Vec<String>,
    pub potions: HashMap<String, potion::Potion>,
    pub potion_names: Vec<String>,
    pub items: HashMap<String, item::Item>,
    pub item_names: Vec<String>,
    pub moves: HashMap<String, r#move::Move>,
    pub move_names: Vec<String>,
    pub natures: HashMap<String, nature::Nature>,
    pub nature_names: Vec<String>,
    pub pokemon: HashMap<String, pokemon::Pokemon>,
    pub pokemon_by_api_id: HashMap<PokemonApiId, pokemon::Pokemon>,
    pub pokemon_names: Vec<String>,
    pub status_effects: HashMap<String, status_effect::StatusEffect>,
    pub status_effects_names: Vec<String>,
    pub weather: HashMap<String, weather::Weather>,
    pub weather_names: Vec<String>,

    // This works on the assumption that issues will be resolved ASAP by the respective data maintainers, so it should never take up much space
    pub issues: Option<IssueStorage>,
}

pub struct MultiSourceGameData {
    pub base_data: Arc<GameData>,
    pub custom_data: Arc<HashMap<i64, GameData>>,

    pub type_efficiency: Arc<type_efficiency::TypeEfficiency>,
}

impl MultiSourceGameData {
    pub async fn get(
        &self,
        guild_id: Option<GuildId>,
        user_id: UserId,
        database: &Pool<Sqlite>,
    ) -> &GameData {
        let custom_data_id = if let Some(guild_id) = guild_id {
            let guild_id = guild_id.get() as i64;
            if let Ok(record) =
                sqlx::query!("SELECT data_source_id FROM guild WHERE id = ?", guild_id)
                    .fetch_one(database)
                    .await
            {
                if let Some(data_source_id) = record.data_source_id {
                    data_source_id
                } else {
                    guild_id
                }
            } else {
                guild_id
            }
        } else {
            let user_id = user_id.get() as i64;
            if let Ok(record) =
                sqlx::query!("SELECT last_data_source_id FROM user WHERE id = ?", user_id)
                    .fetch_one(database)
                    .await
            {
                if let Some(last_data_source) = record.last_data_source_id {
                    last_data_source
                } else {
                    0
                }
            } else {
                0
            }
        };

        if let Some(data) = self.custom_data.get(&custom_data_id) {
            data
        } else {
            &self.base_data
        }
    }

    pub async fn get_by_context(&self, ctx: &poise::Context<'_, Data, Error>) -> &GameData {
        self.get(ctx.guild_id(), ctx.author().id, &ctx.data().database)
            .await
    }

    pub async fn get_by_interaction(
        &self,
        interaction: &&ComponentInteraction,
        database: &Pool<Sqlite>,
    ) -> &GameData {
        self.get(interaction.guild_id, interaction.user.id, database)
            .await
    }
}
