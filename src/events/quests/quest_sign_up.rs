use crate::data::Data;
use crate::Error;
use serenity::client::Context;
use serenity::model::prelude::message_component::MessageComponentInteraction;

pub async fn quest_sign_up(
    context: &Context,
    interaction: &&MessageComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let channel_id = interaction.channel_id.0 as i64;
    interaction
        .create_interaction_response(context, |f| f)
        .await?;

    Ok(())
}

async fn persist_signup(data: &Data, channel_id: i64, character_id: i64) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::events::quests::quest_sign_up::persist_signup;
    use crate::{database_helpers, Error};
    use chrono::Utc;
    use more_asserts::{assert_ge, assert_le};
    use sqlx::{Pool, Sqlite};

    #[sqlx::test]
    async fn sign_up(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let channel_id = 100;
        let creator_id = 200;
        let guild_id = 300;
        let bot_message_id = 400;
        let character_id = 500;
        let character_name = String::from("test");

        database_helpers::create_mock::guild(&data.database, guild_id).await;
        database_helpers::create_mock::user(&data.database, creator_id).await;
        database_helpers::create_mock::quest(
            &data.database,
            channel_id,
            guild_id,
            creator_id,
            bot_message_id,
        )
        .await;
        database_helpers::create_mock::character(
            &data.database,
            guild_id,
            creator_id,
            character_id,
            character_name,
        )
        .await;

        let timestamp_before = Utc::now().timestamp();
        persist_signup(&data, guild_id, character_id).await?;
        let timestamp_after = Utc::now().timestamp();

        let signups =
            sqlx::query!("SELECT quest_id, character_id, creation_timestamp FROM quest_signup")
                .fetch_all(&data.database)
                .await?;

        let signup = signups.first().unwrap();
        assert_eq!(channel_id, signup.quest_id);
        assert_eq!(character_id, signup.character_id);
        assert_le!(timestamp_before, signup.creation_timestamp);
        assert_ge!(timestamp_after, signup.creation_timestamp);

        Ok(())
    }
}
