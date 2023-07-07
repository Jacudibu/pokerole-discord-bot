use crate::commands::{Context, Error};
use crate::data::Data;
use sqlx::{Pool, Sqlite};

#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn create_quest(ctx: Context<'_>) -> Result<(), Error> {
    create_quest_impl(
        ctx.data(),
        ctx.channel_id().0 as i64,
        ctx.author().id.0 as i64,
    )
    .await
}

async fn create_quest_impl(db: &Data, channel_id: i64, creator_id: i64) -> Result<(), Error> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::commands::quests::create_quest::create_quest_impl;
    use crate::data::Data;
    use crate::{game_data, Error};
    use sqlx::{Pool, Sqlite};
    use std::sync::Arc;

    async fn create_mock_data(db: Pool<Sqlite>) -> Data {
        // TODO: Only initialize game data once every cargo test run
        let game_data = game_data::parser::initialize_data().await;
        Data::new(db, Arc::new(game_data)).await
    }

    #[sqlx::test]
    async fn create_quest(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = create_mock_data(db).await;
        let channel_id = 100;
        let creator_id = 200;
        let timestamp_before = i64::min_value(); // FIXME
        create_quest_impl(&data, channel_id, creator_id).await?;
        let timestamp_after = i64::max_value(); // FIXME

        let quests = sqlx::query!(
            "SELECT guild_id, creator_id, creation_timestamp, completion_timestamp FROM quest"
        )
        .fetch_all(&data.database)
        .await?;

        let quest = quests.first().unwrap();
        assert_eq!(1, quest.creator_id);
        assert_eq!(1, quest.guild_id);
        assert!((timestamp_before..timestamp_after).contains(&quest.creation_timestamp));
        assert_eq!(None, quest.completion_timestamp);

        Ok(())
    }
}
