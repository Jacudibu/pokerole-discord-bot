use crate::data::Data;
use crate::enums::QuestParticipantSelectionMechanism;
use crate::{helpers, Error};
use chrono::Utc;
use serenity::client::Context;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::model::prelude::InteractionResponseType;
use std::str::FromStr;

pub async fn quest_sign_up(
    context: &Context,
    interaction: &&MessageComponentInteraction,
    data: &Data,
    args: Vec<&str>,
) -> Result<(), Error> {
    let guild_id = interaction
        .guild_id
        .expect("Command should be guild_only")
        .0 as i64;
    let user_id = interaction.user.id.0 as i64;
    let channel_id = interaction.channel_id.0 as i64;

    let available_characters = sqlx::query!(
        "SELECT id, name FROM character WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id
    )
    .fetch_all(&data.database)
    .await?;

    if args.len() == 2 {
        let character_id = i64::from_str(args[0])?;
        let timestamp = i64::from_str(args[1])?;

        if available_characters.iter().all(|x| x.id != character_id) {
            // TODO: Handle Invalid button input. That's a biiiig red flag!
            return Ok(());
        }

        return process_signup(
            context,
            interaction,
            InteractionResponseType::UpdateMessage,
            data,
            channel_id,
            character_id,
            timestamp,
        )
        .await;
    }

    let timestamp = Utc::now().timestamp();
    if available_characters.len() == 1 {
        return process_signup(
            context,
            interaction,
            InteractionResponseType::ChannelMessageWithSource,
            data,
            channel_id,
            available_characters[0].id,
            timestamp,
        )
        .await;
    }

    interaction
        .create_interaction_response(context, |interaction_response| {
            interaction_response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data.ephemeral(true)
                        .content("Which character would you like to sign up?")
                        .components(|components| {
                            components.create_action_row(|row| {
                                for x in available_characters {
                                    row.add_button(helpers::create_button(
                                        x.name.as_str(),
                                        &format!("quest-sign-up_{}_{}", x.id, timestamp),
                                        false,
                                    ));
                                }
                                row
                            })
                        })
                })
        })
        .await?;
    Ok(())
}

async fn process_signup(
    context: &Context,
    interaction: &&MessageComponentInteraction,
    response_type: InteractionResponseType,
    data: &Data,
    channel_id: i64,
    character_id: i64,
    timestamp: i64,
) -> Result<(), Error> {
    let result = persist_signup(data, channel_id, character_id, timestamp).await;
    // TODO: Error Handling

    interaction
        .create_interaction_response(context, |response| {
            response
                .kind(response_type)
                .interaction_response_data(|data| {
                    data.ephemeral(true)
                        .content("Successfully signed up!")
                        .components(|components| components)
                })
        })
        .await?;

    let quest_record = sqlx::query!(
        "SELECT bot_message_id, maximum_participant_count, participant_selection_mechanism FROM quest WHERE channel_id = ?",
        channel_id
    )
    .fetch_one(&data.database)
    .await?;

    let text = helpers::generate_quest_post_message_content(
        data,
        channel_id,
        quest_record.maximum_participant_count,
        QuestParticipantSelectionMechanism::from_repr(quest_record.participant_selection_mechanism)
            .expect("Should always be valid!"),
    )
    .await?;

    let message = context
        .http
        .get_message(channel_id as u64, quest_record.bot_message_id as u64)
        .await;
    if let Ok(mut message) = message {
        message
            .edit(context, |edit| {
                edit.content(text)
                    .components(|components| helpers::create_quest_signup_buttons(components))
            })
            .await?;
    }

    Ok(())
}

async fn persist_signup(
    data: &Data,
    channel_id: i64,
    character_id: i64,
    timestamp: i64,
) -> Result<(), String> {
    let result = sqlx::query!(
        "INSERT INTO quest_signup (quest_id, character_id, timestamp) VALUES (?, ?, ?)",
        channel_id,
        character_id,
        timestamp
    )
    .execute(&data.database)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(())
            } else {
                Err(String::from("Unable to persist quest signup!"))
            }
        }
        Err(e) => Err(format!("**Something went wrong!**\n{}", e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::enums::QuestParticipantSelectionMechanism;
    use crate::events::quests::quest_sign_up::persist_signup;
    use crate::{database_helpers, Error};
    use chrono::Utc;
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
            5,
            QuestParticipantSelectionMechanism::Random,
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

        let timestamp = Utc::now().timestamp();
        persist_signup(&data, channel_id, character_id, timestamp).await?;

        let signups = sqlx::query!("SELECT quest_id, character_id, timestamp FROM quest_signup")
            .fetch_all(&data.database)
            .await?;

        let signup = signups.first().unwrap();
        assert_eq!(channel_id, signup.quest_id);
        assert_eq!(character_id, signup.character_id);
        assert_eq!(timestamp, signup.timestamp);

        Ok(())
    }
}
