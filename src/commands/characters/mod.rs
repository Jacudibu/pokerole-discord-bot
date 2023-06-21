use core::fmt;
use std::fmt::{Formatter};
use poise::Command;
use regex::Regex;
use serenity::model::id::ChannelId;
use crate::commands::{Context, send_error};
use crate::{emoji, Error};
use crate::cache::CharacterCacheItem;
use crate::data::Data;
use crate::enums::MysteryDungeonRank;

mod initialize_character;
mod reward_money;
mod reward_experience;
mod initialize_guild;
mod complete_quest;
mod initialize_character_post;
mod give_money;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec!(
        complete_quest::complete_quest(),
        give_money::give_money(),
        initialize_character::initialize_character(),
        initialize_character_post::initialize_character_post(),
        initialize_guild::initialize_guild(),
        reward_experience::reward_experience(),
        reward_money::reward_money(),
    )
}

pub async fn send_stale_data_error<'a>(ctx: &Context<'a>) -> Result<(), Error> {
    send_error(ctx, "Something went wrong!
You hit an absolute edge case where the value has been updated by someone else while this command has been running.
If this seriously ever happens and/or turns into a problem, let me know. For now... try again? :'D
You can copy the command string either by just pressing the up key inside the text field on pc."
    ).await
}

pub async fn update_character_post<'a>(ctx: &Context<'a>, id: i64) -> Result<(), Error> {
    if let Some(result) = build_character_string(ctx, id).await {
        let message = ctx.serenity_context().http.get_message(result.1 as u64, result.2 as u64).await;
        if let Ok(mut message) = message {
            message.edit(ctx, |f| f.content(result.0)).await?;
        }
    }

    Ok(())
}

// TODO: we really should just change this to a query_as thingy...
pub async fn build_character_string<'a>(ctx: &Context<'a>, character_id: i64) -> Option<(String, i64, i64)> {
    let entry = sqlx::query!(
                "SELECT name, experience, money, completed_quest_count, stat_message_id, stat_channel_id \
                FROM character WHERE id = ? \
                ORDER BY rowid \
                LIMIT 1",
                character_id,
            )
        .fetch_one(&ctx.data().database)
        .await;

    match entry {
        Ok(entry) => {
            let level = entry.experience / 100 + 1;
            let experience = entry.experience % 100;
            let rank = MysteryDungeonRank::from_level(level as u8);

            Some((format!("\
## {} {}
{} {}
**Level**: {} `({} / 100)`
Completed Quests: {}
",
                         rank.emoji_string(), entry.name, entry.money, emoji::POKE_COIN, level, experience, entry.completed_quest_count)
                  , entry.stat_channel_id, entry.stat_message_id))
        }
        Err(_) => None,
    }
}


pub enum ActionType {
    Initialization,
    Reward,
    TradeOutgoing,
    TradeIncoming,
    Undo,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ActionType::Initialization => "🌟 [Init]",
            ActionType::Reward => "✨ [Reward]",
            ActionType::TradeOutgoing => "➡️ [Trade]",
            ActionType::TradeIncoming => "⬅️ [Trade]",
            ActionType::Undo => "↩️ [Undo]",
        })
    }
}

pub async fn log_action<'a>(action_type: ActionType, ctx: &Context<'a>, message: &str) -> Result<(), Error> {
    let guild_id = ctx.guild_id();
    if guild_id.is_none() {
        return Ok(());
    }

    let guild_id = guild_id.expect("should only be called in guild_only").0 as i64;
    let record = sqlx::query!("SELECT action_log_channel_id FROM guild WHERE id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await;

    if let Ok(record) = record {
        if let Some(action_log_channel_id) = record.action_log_channel_id {
            let channel_id= ChannelId::from(action_log_channel_id as u64);
            channel_id.send_message(ctx, |f| f
                .content(std::format!("{} {} (triggered by {})", action_type, message, ctx.author()))
                .allowed_mentions(|mentions| mentions.empty_users())
            ).await?;
        }
    }

    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct CharacterWithNumericValue {
    id: i64,
    user_id: i64,
    name: String,
    value: i64
}

pub async fn change_character_stat<'a>(ctx: &Context<'a>, database_column: &str, name: &String, amount: i64, action_type: ActionType) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Commands using this function are marked as guild_only").0;
    let character = parse_user_input_to_character(ctx, guild_id, name).await;
    match character {
        Some(character) => {
            change_character_stat_internal(ctx, database_column, character, amount, action_type).await
        }
        None => {
            send_error(ctx, format!("Unable to find a character named {}", name).as_str()).await
        }
    }
}

async fn change_character_stat_internal<'a>(ctx: &Context<'a>, database_column: &str, character: CharacterCacheItem, amount: i64, action_type: ActionType) -> Result<(), Error> {
    let record = sqlx::query_as::<_, CharacterWithNumericValue>(
        format!("SELECT id, user_id, name, {} as value FROM character WHERE name = ? AND guild_id = ?", database_column).as_str())
        .bind(&character.name)
        .bind(character.guild_id as i64)
        .fetch_one(&ctx.data().database)
        .await;

    match record {
        Ok(record) => {
            let new_value = record.value + amount;
            let result = sqlx::query(
                format!("UPDATE character SET {} = ? WHERE id = ? AND {} = ?", database_column, database_column).as_str())
                .bind(new_value)
                .bind(record.id)
                .bind(record.value)
                .execute(&ctx.data().database).await?;

            if result.rows_affected() != 1 {
                return send_stale_data_error(ctx).await;
            }

            update_character_post(ctx, record.id).await?;
            let action = if database_column == "money" {
                emoji::POKE_COIN
            } else {
                database_column
            };
            let added_or_removed: &str;
            let to_or_from: &str;
            if amount > 0 {
                added_or_removed = "Added";
                to_or_from = "to";
            } else {
                added_or_removed = "Removed";
                to_or_from = "from";
            }

            log_action(action_type, ctx, format!("{} {} {} {} {}", added_or_removed, amount.abs(), action, to_or_from, record.name).as_str()).await
        }
        Err(_) => {
            send_error(ctx, format!("Unable to find a character named {}.\n**Internal cache must be out of date. Please let me know if this ever happens.**", character.name).as_str()).await
        }
    }
}

pub async fn parse_user_input_to_character<'a>(ctx: &Context<'a>, guild_id: u64, text: &str) -> Option<CharacterCacheItem> {
    let characters = ctx.data().cache.get_characters().await;
    for x in &characters {
        if x.guild_id == guild_id && text == x.get_autocomplete_name() {
            return Some(x.clone());
        }
    }

    // User didn't use an autocomplete name :<
    let lowercase_input = text.to_lowercase();
    let name_matches: Vec<&CharacterCacheItem> = characters.iter()
        .filter(|x| x.guild_id == guild_id && x.name.to_lowercase() == lowercase_input)
        .collect();

    if name_matches.len() != 1 {
        None
    } else {
        name_matches.get(0).cloned().cloned()
    }
}

pub fn validate_user_input<'a>(text: &str) -> Result<(), &'a str> {
    if text.len() > 30 {
        return Err("Query string too long!");
    }

    // TODO: Move that thing into some static context
    let regex = Regex::new("^[a-zA-Z0-9]*$").unwrap();
    if regex.is_match(text) {
        Ok(())
    } else {
        Err("Failed to validate input!")
    }
}
