use crate::commands::{send_error, Error};
use crate::shared::data::Data;
use crate::shared::enums::QuestParticipantSelectionMechanism;
use crate::shared::utility::quest_message_utils;
use crate::shared::PoiseContext;
use chrono::Utc;
use poise::CreateReply;
use serenity::all::{GetMessages, MessageType};

#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn create_quest(
    ctx: PoiseContext<'_>,
    #[min = 1_i64] max_participants: i64,
    selection_mechanism: QuestParticipantSelectionMechanism,
) -> Result<(), Error> {
    let reply = ctx
        .send(CreateReply::default().content("Creating Quest..."))
        .await?;
    let reply_message = reply.message().await?;
    let channel_id = ctx.channel_id().get() as i64;

    let messages = ctx
        .guild_channel()
        .await
        .unwrap()
        .messages(ctx, GetMessages::new().limit(50))
        .await
        .unwrap();

    if messages.len() > 40 {
        let _ = reply_message.delete(&ctx).await;
        return send_error(&ctx, "There have already been over 40 messages sent within this channel. Are you sure you are using this command in a freshly created forum post?").await;
    }

    let oldest_message_inside_channel = messages
        .iter()
        .min_by(|a, b| a.timestamp.cmp(&b.timestamp))
        .expect("There should be at least one message in this forum?");

    let result = create_quest_impl(
        ctx.data(),
        ctx.guild_id().expect("Command is guild_only").get() as i64,
        channel_id,
        ctx.author().id.get() as i64,
        reply_message.id.get() as i64,
        max_participants,
        selection_mechanism,
        oldest_message_inside_channel.id.get() as i64,
    )
    .await;

    match result {
        Ok(_) => {
            let (text, too_many_signups) =
                quest_message_utils::generate_quest_post_message_content(
                    ctx.serenity_context(),
                    ctx.data(),
                    channel_id,
                    max_participants,
                    selection_mechanism,
                )
                .await?;
            reply
                .edit(
                    ctx,
                    CreateReply::default().content(text).components(
                        quest_message_utils::create_quest_signup_buttons(
                            selection_mechanism,
                            too_many_signups,
                        ),
                    ),
                )
                .await?;

            let _ = reply_message.pin(&ctx).await;
            let _ = oldest_message_inside_channel.pin(&ctx).await;

            for message in ctx
                .guild_channel()
                .await
                .unwrap()
                .messages(ctx, GetMessages::new())
                .await
                .unwrap()
            {
                if message.kind == MessageType::PinsAdd {
                    let _ = message.delete(&ctx).await;
                }
            }

            Ok(())
        }
        Err(e) => {
            let text = if e.contains("UNIQUE constraint failed") {
                "A quest was already created for this channel!"
            } else {
                e.as_str()
            };

            reply
                .edit(ctx, CreateReply::default().content(text))
                .await?;
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
    max_participants: i64,
    selection_mechanism: QuestParticipantSelectionMechanism,
    quest_description_message_id: i64,
) -> Result<(), String> {
    let timestamp = Utc::now().timestamp();

    let result = sqlx::query!("INSERT INTO quest (guild_id, channel_id, creator_id, bot_message_id, creation_timestamp, maximum_participant_count, participant_selection_mechanism, quest_description_message_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        guild_id, channel_id, creator_id, bot_message_id, timestamp, max_participants, selection_mechanism, quest_description_message_id
    )
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
    use crate::commands::quest_commands::create_quest::create_quest_impl;
    use crate::shared::enums::QuestParticipantSelectionMechanism;
    use crate::{database_mocks, Error};
    use chrono::Utc;
    use more_asserts::{assert_ge, assert_le};
    use sqlx::{Pool, Sqlite};

    #[sqlx::test]
    async fn create_quest(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_mocks::create_mock::data(db).await;
        let channel_id = 100;
        let creator_id = 200;
        let guild_id = 300;
        let bot_message_id = 400;
        let max_participants = 5;
        let mechanism = QuestParticipantSelectionMechanism::FirstComeFirstServe;
        let quest_description_message_id = 700;

        database_mocks::create_mock::guild(&data.database, guild_id).await;
        database_mocks::create_mock::user(&data.database, creator_id).await;
        let timestamp_before = Utc::now().timestamp();
        create_quest_impl(
            &data,
            guild_id,
            channel_id,
            creator_id,
            bot_message_id,
            max_participants,
            mechanism,
            quest_description_message_id,
        )
        .await?;
        let timestamp_after = Utc::now().timestamp();

        let quests = sqlx::query!(
            "SELECT guild_id, creator_id, channel_id, creation_timestamp, completion_timestamp, maximum_participant_count, participant_selection_mechanism FROM quest"
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
        assert_eq!(max_participants, quest.maximum_participant_count);
        assert_eq!(
            mechanism,
            QuestParticipantSelectionMechanism::from_repr(quest.participant_selection_mechanism)
                .unwrap()
        );

        Ok(())
    }

    #[sqlx::test]
    async fn create_quest_called_twice(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_mocks::create_mock::data(db).await;
        let channel_id = 100;
        let creator_id = 200;
        let guild_id = 300;
        let bot_message_id = 400;
        let max_participants = 5;
        let selection_mechanism = QuestParticipantSelectionMechanism::FirstComeFirstServe;
        let quest_description_message_id = 700;

        database_mocks::create_mock::guild(&data.database, guild_id).await;
        database_mocks::create_mock::user(&data.database, creator_id).await;

        create_quest_impl(
            &data,
            guild_id,
            channel_id,
            creator_id,
            bot_message_id,
            max_participants,
            selection_mechanism,
            quest_description_message_id,
        )
        .await?;
        let result = create_quest_impl(
            &data,
            guild_id,
            channel_id,
            creator_id,
            bot_message_id,
            max_participants,
            selection_mechanism,
            quest_description_message_id,
        )
        .await;

        assert!(result.is_err());

        Ok(())
    }
}
