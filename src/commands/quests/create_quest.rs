use crate::commands::{Context, Error};
use crate::data::Data;
use crate::enums::QuestParticipantSelectionMechanism;
use crate::helpers;
use chrono::Utc;

#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn create_quest(
    ctx: Context<'_>,
    #[min = 1_i64] max_participants: i64,
    selection_mechanism: QuestParticipantSelectionMechanism,
) -> Result<(), Error> {
    let reply = ctx.send(|f| f.content("Creating Quest...")).await?;
    let message_id = reply.message().await?.id;
    let channel_id = ctx.channel_id().0 as i64;

    let result = create_quest_impl(
        ctx.data(),
        ctx.guild_id().expect("Command is guild_only").0 as i64,
        channel_id,
        ctx.author().id.0 as i64,
        message_id.0 as i64,
        max_participants,
        selection_mechanism,
    )
    .await;

    match result {
        Ok(_) => {
            let text = helpers::generate_quest_post_message_content(
                ctx.data(),
                channel_id,
                max_participants,
                selection_mechanism,
            )
            .await?;
            reply
                .edit(ctx, |edit| {
                    edit.content(text)
                        .components(|components| helpers::create_quest_signup_buttons(components))
                })
                .await?;
            Ok(())
        }
        Err(e) => {
            let text = if e.contains("UNIQUE constraint failed") {
                "A quest was already created for this channel!"
            } else {
                e.as_str()
            };

            reply.edit(ctx, |f| f.content(text)).await?;
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
) -> Result<(), String> {
    let timestamp = Utc::now().timestamp();

    let result = sqlx::query!("INSERT INTO quest (guild_id, channel_id, creator_id, bot_message_id, creation_timestamp, maximum_participant_count, participant_selection_mechanism) VALUES (?, ?, ?, ?, ?, ?, ?)",
        guild_id, channel_id, creator_id, bot_message_id, timestamp, max_participants, selection_mechanism
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
    use crate::commands::quests::create_quest::create_quest_impl;
    use crate::enums::QuestParticipantSelectionMechanism;
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
        let max_participants = 5;
        let mechanism = QuestParticipantSelectionMechanism::FirstComeFirstServe;

        database_helpers::create_mock::guild(&data.database, guild_id).await;
        database_helpers::create_mock::user(&data.database, creator_id).await;
        let timestamp_before = Utc::now().timestamp();
        create_quest_impl(
            &data,
            guild_id,
            channel_id,
            creator_id,
            bot_message_id,
            max_participants,
            mechanism,
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
        let data = database_helpers::create_mock::data(db).await;
        let channel_id = 100;
        let creator_id = 200;
        let guild_id = 300;
        let bot_message_id = 400;
        let max_participants = 5;
        let selection_mechanism = QuestParticipantSelectionMechanism::FirstComeFirstServe;

        database_helpers::create_mock::guild(&data.database, guild_id).await;
        database_helpers::create_mock::user(&data.database, creator_id).await;

        create_quest_impl(
            &data,
            guild_id,
            channel_id,
            creator_id,
            bot_message_id,
            max_participants,
            selection_mechanism,
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
        )
        .await;

        assert!(result.is_err());

        Ok(())
    }
}
