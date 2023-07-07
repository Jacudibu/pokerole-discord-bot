use crate::commands::{Context, Error};
use crate::data::Data;
use crate::helpers;
use chrono::Utc;
use serenity::model::prelude::component::ButtonStyle;

#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn create_quest(ctx: Context<'_>) -> Result<(), Error> {
    let reply = ctx.send(|f| f.content("Creating Quest...")).await?;
    let message_id = reply.message().await?.id;

    let result = create_quest_impl(
        ctx.data(),
        ctx.guild_id().expect("Command is guild_only").0 as i64,
        ctx.channel_id().0 as i64,
        ctx.author().id.0 as i64,
        message_id.0 as i64,
    )
    .await;

    match result {
        Ok(_) => {
            reply
                .edit(ctx, |edit| {
                    edit.content("Quest created!").components(|components| {
                        components.create_action_row(|action_row| {
                            action_row.add_button(helpers::create_styled_button(
                                "Sign up!",
                                "quest-sign-up",
                                false,
                                ButtonStyle::Success,
                            ))
                        })
                    })
                })
                .await?;
            Ok(())
        }
        Err(e) => {
            reply.edit(ctx, |f| f.content(e.as_str())).await?;
            Ok(())
        }
    }
}

async fn create_quest_impl(
    data: &Data,
    guild_id: i64,
    channel_id: i64,
    creator_id: i64,
    bot_message_id: i64,
) -> Result<(), String> {
    let timestamp = Utc::now().timestamp();

    let result = sqlx::query!("INSERT INTO quest (guild_id, channel_id, creator_id, bot_message_id, creation_timestamp) VALUES (?, ?, ?, ?, ?)", guild_id, channel_id, creator_id, bot_message_id, timestamp)
        .execute(&data.database).await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(())
            } else {
                Err(String::from("Unable to persist quest entry!"))
            }
        }
        Err(e) => Err(format!("**Something went wrong!**\n{}", e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::quests::create_quest::create_quest_impl;
    use crate::{database_helpers, Error};
    use chrono::Utc;
    use more_asserts::{assert_ge, assert_le};
    use sqlx::{Pool, Sqlite};

    #[sqlx::test]
    async fn create_quest(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let channel_id = 100;
        let creator_id = 200;
        let guild_id = 300;
        let bot_message_id = 400;

        database_helpers::create_mock::guild(&data.database, guild_id).await;
        database_helpers::create_mock::user(&data.database, creator_id).await;
        let timestamp_before = Utc::now().timestamp();
        create_quest_impl(&data, guild_id, channel_id, creator_id, bot_message_id).await?;
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

    #[sqlx::test]
    async fn create_quest_called_twice(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let channel_id = 100;
        let creator_id = 200;
        let guild_id = 300;
        let bot_message_id = 400;

        database_helpers::create_mock::guild(&data.database, guild_id).await;
        database_helpers::create_mock::user(&data.database, creator_id).await;

        create_quest_impl(&data, guild_id, channel_id, creator_id, bot_message_id).await?;
        let result =
            create_quest_impl(&data, guild_id, channel_id, creator_id, bot_message_id).await;

        assert!(result.is_err());

        Ok(())
    }
}
