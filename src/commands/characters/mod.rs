use crate::cache::CharacterCacheItem;
use crate::commands::{
    handle_error_during_message_edit, parse_character_names, send_error,
    BuildUpdatedStatMessageStringResult, Context,
};
use crate::data::Data;
use crate::enums::MysteryDungeonRank;
use crate::{emoji, Error};
use core::fmt;
use poise::Command;
use regex::Regex;
use serenity::all::{CreateAllowedMentions, CreateMessage, EditMessage, MessageId};
use serenity::model::id::ChannelId;
use sqlx::{Pool, Sqlite};
use std::fmt::Formatter;

mod give_money;
mod initialize_character;
mod initialize_character_post;
mod initialize_guild;
mod reward_experience;
mod reward_money;
mod spar;
mod upgrade_backpack;

const DEFAULT_BACKPACK_SLOTS: i64 = 6;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![
        give_money::give_money(),
        initialize_character::initialize_character(),
        initialize_character_post::initialize_character_post(),
        initialize_guild::initialize_guild(),
        reward_experience::reward_experience(),
        reward_money::reward_money(),
        upgrade_backpack::upgrade_backpack(),
        spar::spar(),
    ]
}

pub async fn send_stale_data_error<'a>(ctx: &Context<'a>) -> Result<(), Error> {
    send_error(ctx, "Something went wrong!
You hit an absolute edge case where the value has been updated by someone else while this command has been running.
If this seriously ever happens and/or turns into a problem, let me know. For now... try again? :'D
You can copy the command string either by just pressing the up key inside the text field on pc."
    ).await
}

pub async fn update_character_post<'a>(ctx: &Context<'a>, id: i64) {
    if let Some(result) = build_character_string(&ctx.data().database, id).await {
        let message = ctx
            .serenity_context()
            .http
            .get_message(
                ChannelId::from(result.stat_channel_id as u64),
                MessageId::from(result.stat_message_id as u64),
            )
            .await;
        if let Ok(mut message) = message {
            if let Err(e) = message
                .edit(ctx, EditMessage::new().content(&result.message))
                .await
            {
                handle_error_during_message_edit(ctx, e, message, result.message, result.name)
                    .await;
            }
        }
    }
}

async fn count_completed_quests<'a>(database: &Pool<Sqlite>, character_id: i64) -> i32 {
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM quest_completion WHERE character_id = ?",
        character_id
    )
    .fetch_optional(database)
    .await;

    if let Ok(Some(record)) = result {
        record.count
    } else {
        0
    }
}

pub async fn build_character_string(
    database: &Pool<Sqlite>,
    character_id: i64,
) -> Option<BuildUpdatedStatMessageStringResult> {
    let entry = sqlx::query!(
        "SELECT name, experience, money, stat_message_id, stat_channel_id, backpack_upgrade_count, total_spar_count, weekly_spar_count \
                FROM character WHERE id = ? \
                ORDER BY rowid \
                LIMIT 1",
        character_id,
    )
    .fetch_one(database)
    .await;

    let completed_quest_count = count_completed_quests(database, character_id).await;

    match entry {
        Ok(entry) => {
            let level = entry.experience / 100 + 1;
            let experience = entry.experience % 100;
            let rank = MysteryDungeonRank::from_level(level as u8);

            Some(BuildUpdatedStatMessageStringResult {
                message: format!(
                    "\
## {} {}
{} {}
**Level**: {} `({} / 100)`
Completed Quests: {}
Backpack Slots: {}
Sparring Sessions: {} ({} this week)
",
                    rank.emoji_string(),
                    entry.name,
                    entry.money,
                    emoji::POKE_COIN,
                    level,
                    experience,
                    completed_quest_count,
                    entry.backpack_upgrade_count + DEFAULT_BACKPACK_SLOTS,
                    entry.total_spar_count,
                    entry.weekly_spar_count
                ),
                name: entry.name,
                stat_channel_id: entry.stat_channel_id,
                stat_message_id: entry.stat_message_id,
            })
        }
        Err(_) => None,
    }
}

pub enum ActionType {
    Initialization,
    Reward,
    Payment,
    BackpackUpgrade,
    TradeOutgoing,
    TradeIncoming,
    WalletChange,
    WalletPayment,
    WalletWithdrawal,
    Undo,
    Spar,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ActionType::Initialization => "🌟 [Init]",
            ActionType::Reward => "✨ [Reward]",
            ActionType::BackpackUpgrade => "🎒 [Upgrade]",
            ActionType::Payment => "💰 [Payment]",
            ActionType::TradeOutgoing => "➡️ [Trade]",
            ActionType::TradeIncoming => "⬅️ [Trade]",
            ActionType::WalletChange => "👛 [Wallet]",
            ActionType::WalletPayment => "👛⬅️ [Payment]",
            ActionType::WalletWithdrawal => "👛➡️ [Withdrawal]",
            ActionType::Undo => "↩️ [Undo]",
            ActionType::Spar => "🤺 [Spar]",
        })
    }
}

pub async fn log_action<'a>(
    action_type: &ActionType,
    ctx: &Context<'a>,
    message: &str,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id();
    if guild_id.is_none() {
        return Ok(());
    }

    let guild_id = guild_id.expect("should only be called in guild_only").get() as i64;
    let record = sqlx::query!(
        "SELECT action_log_channel_id FROM guild WHERE id = ?",
        guild_id
    )
    .fetch_one(&ctx.data().database)
    .await;

    if let Ok(record) = record {
        if let Some(action_log_channel_id) = record.action_log_channel_id {
            let channel_id = ChannelId::from(action_log_channel_id as u64);
            channel_id
                .send_message(
                    ctx,
                    CreateMessage::new()
                        .content(std::format!(
                            "{} {} (triggered by {})",
                            action_type,
                            message,
                            ctx.author()
                        ))
                        .allowed_mentions(CreateAllowedMentions::new().empty_users()),
                )
                .await?;
        }
    }

    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct EntityWithNameAndNumericValue {
    pub id: i64,
    pub name: String,
    pub value: i64,
}

pub async fn change_character_stat<'a>(
    ctx: &Context<'a>,
    database_column: &str,
    names: &Vec<String>,
    amount: i64,
    action_type: ActionType,
) -> Result<Vec<CharacterCacheItem>, String> {
    let guild_id = ctx
        .guild_id()
        .expect("Commands using this function are marked as guild_only")
        .get();

    match parse_character_names(ctx, guild_id, names).await {
        Ok(characters) => {
            for x in &characters {
                let _ = change_character_stat_after_validation(
                    ctx,
                    database_column,
                    x,
                    amount,
                    &action_type,
                )
                .await;
            }
            Ok(characters)
        }
        Err(error) => Err(error),
    }
}

pub async fn change_character_stat_after_validation<'a>(
    ctx: &Context<'a>,
    database_column: &str,
    character: &CharacterCacheItem,
    amount: i64,
    action_type: &ActionType,
) -> Result<(), Error> {
    ctx.defer().await?;
    let record = sqlx::query_as::<_, EntityWithNameAndNumericValue>(
        format!(
            "SELECT id, name, {} as value FROM character WHERE id = ?",
            database_column
        )
        .as_str(),
    )
    .bind(character.id)
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
                .execute(&ctx.data().database).await;

            if result.is_err() || result.unwrap().rows_affected() != 1 {
                return send_stale_data_error(ctx).await
            }

            update_character_post(ctx, record.id).await;
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

pub fn validate_user_input<'a>(text: &str) -> Result<(), &'a str> {
    if text.len() > 30 {
        return Err("Query string too long!");
    }

    // TODO: Move that thing into some static context
    let regex = Regex::new("^[a-zA-Z0-9\\s]*$").unwrap();
    if regex.is_match(text) {
        Ok(())
    } else {
        Err("Failed to validate input!")
    }
}

pub fn build_character_list(characters: &[CharacterCacheItem]) -> String {
    characters
        .iter()
        .map(|x| x.name.as_str())
        .collect::<Vec<&str>>()
        .join(", ")
}
