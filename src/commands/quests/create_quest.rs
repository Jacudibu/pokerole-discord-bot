use crate::commands::{send_error, Context, Error};
use crate::data::Data;
use chrono::Utc;

#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn create_quest(ctx: Context<'_>) -> Result<(), Error> {
    let result = create_quest_impl(
        ctx.data(),
        ctx.guild_id().expect("Command is guild_only").0 as i64,
        ctx.channel_id().0 as i64,
        ctx.author().id.0 as i64,
    )
    .await;

    match result {
        Ok(_) => {
            ctx.say("Quest created!").await?;
            Ok(())
        }
        Err(e) => send_error(&ctx, e.as_str()).await,
    }
}

async fn create_quest_impl(
    data: &Data,
    guild_id: i64,
    channel_id: i64,
    creator_id: i64,
) -> Result<(), String> {
    let timestamp = Utc::now().timestamp();

    let result = sqlx::query!("INSERT INTO quest (guild_id, channel_id, creator_id, creation_timestamp) VALUES (?, ?, ?, ?)", guild_id, channel_id, creator_id, timestamp)
        .execute(&data.database).await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(())
            } else {
                Err(String::from("Unable to persist quest entry!"))
            }
        }
        Err(e) => Err(format!("Something went wrong: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::quests::create_quest::create_quest_impl;
    use crate::data::Data;
    use crate::{game_data, Error};
    use chrono::Utc;
    use more_asserts::{assert_ge, assert_le};
    use sqlx::{Pool, Sqlite};
    use std::sync::Arc;

    async fn create_mock_data(db: Pool<Sqlite>) -> Data {
        // TODO: Only initialize game data once every cargo test run
        let game_data = game_data::parser::initialize_data().await;
        Data::new(db, Arc::new(game_data)).await
    }

    async fn create_mock_user(db: &Pool<Sqlite>, user_id: i64) {
        let _ = sqlx::query!("INSERT INTO user (id) VALUES (?)", user_id)
            .execute(db)
            .await;
    }
    async fn create_mock_guild(db: &Pool<Sqlite>, guild_id: i64) {
        let _ = sqlx::query!(
            "INSERT INTO guild (id, money, action_log_channel_id) VALUES (?, ?, ?)",
            guild_id,
            0,
            0
        )
        .execute(db)
        .await;
    }

    #[sqlx::test]
    async fn create_quest(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = create_mock_data(db).await;
        let channel_id = 100;
        let creator_id = 200;
        let guild_id = 300;

        create_mock_guild(&data.database, guild_id).await;
        create_mock_user(&data.database, creator_id).await;
        let timestamp_before = Utc::now().timestamp();
        create_quest_impl(&data, guild_id, channel_id, creator_id).await?;
        let timestamp_after = Utc::now().timestamp();

        let quests = sqlx::query!(
            "SELECT guild_id, creator_id, channel_id, creation_timestamp, completion_timestamp FROM quest"
        )
        .fetch_all(&data.database)
        .await?;

        let quest = quests.first().unwrap();
        assert_eq!(creator_id, quest.creator_id);
        assert_eq!(guild_id, quest.guild_id);
        assert_eq!(channel_id, quest.channel_id);
        assert_le!(timestamp_before, quest.creation_timestamp);
        assert_ge!(timestamp_after, quest.creation_timestamp);
        assert_eq!(None, quest.completion_timestamp);

        Ok(())
    }
}
