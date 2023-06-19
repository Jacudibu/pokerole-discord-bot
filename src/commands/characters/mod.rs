use poise::Command;
use serenity::model::id::ChannelId;
use crate::commands::{Context, send_error};
use crate::{emoji, Error};
use crate::data::Data;
use crate::enums::MysteryDungeonRank;

mod initialize_character;
mod reward_money;
mod reward_experience;
mod initialize_guild;
mod complete_quest;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec!(
        complete_quest::complete_quest(),
        initialize_character::initialize_character(),
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

pub async fn log_action<'a>(ctx: &Context<'a>, message: &str) -> Result<(), Error> {
    let guild_id = ctx.guild_id();
    if guild_id.is_none() {
        return Ok(());
    }

    let guild_id = guild_id.expect("should only be called in guild_only").0 as i64;
    let record = sqlx::query!("SELECT action_log_channel_id FROM guild WHERE id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await;

    if let Ok(record) = record {
        let channel_id= ChannelId::from(record.action_log_channel_id as u64);
        channel_id.send_message(ctx, |f| f.content(message)).await?;
    }

    Ok(())
}
