use crate::data::Data;
use crate::{helpers, Error};
use chrono::Utc;
use serenity::all::{
    ComponentInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::builder::CreateActionRow;
use serenity::client::Context;
use std::str::FromStr;

enum MessageType {
    UpdateMessage,
    NewMessage,
}

pub async fn quest_sign_up(
    context: &Context,
    interaction: &ComponentInteraction,
    data: &Data,
    args: Vec<&str>,
) -> Result<(), Error> {
    let guild_id = interaction
        .guild_id
        .expect("Command should be guild_only")
        .get() as i64;
    let user_id = interaction.user.id.get() as i64;
    let channel_id = interaction.channel_id.get() as i64;

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
            MessageType::UpdateMessage,
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
            MessageType::NewMessage,
            data,
            channel_id,
            available_characters[0].id,
            timestamp,
        )
        .await;
    }

    let character_buttons = available_characters
        .iter()
        .map(|x| {
            helpers::create_button(
                x.name.as_str(),
                &format!("quest-sign-up_{}_{}", x.id, timestamp),
                false,
            )
        })
        .collect();

    interaction
        .create_response(
            context,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content("Which character would you like to sign up?")
                    .components(vec![CreateActionRow::Buttons(character_buttons)]),
            ),
        )
        .await?;

    Ok(())
}

async fn process_signup(
    context: &Context,
    interaction: &ComponentInteraction,
    response_type: MessageType,
    data: &Data,
    channel_id: i64,
    character_id: i64,
    timestamp: i64,
) -> Result<(), Error> {
    let timestamp = if Utc::now().timestamp() - timestamp > 60 {
        Utc::now().timestamp()
    } else {
        timestamp
    };

    let result = persist_signup(data, channel_id, character_id, timestamp).await;

    let text = if let Some(error) = result.err() {
        if error.contains("UNIQUE constraint failed") {
            String::from("Seems like you are already signed up!")
        } else {
            error
        }
    } else {
        String::from("Successfully signed up!")
    };

    let message = CreateInteractionResponseMessage::new()
        .ephemeral(true)
        .content(text)
        .components(Vec::new());

    match response_type {
        MessageType::UpdateMessage => {
            interaction.create_response(context, CreateInteractionResponse::UpdateMessage(message))
        }

        MessageType::NewMessage => {
            interaction.create_response(context, CreateInteractionResponse::Message(message))
        }
    }
    .await?;

    helpers::update_quest_message(context, data, channel_id).await?;

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
            &data,
            guild_id,
            creator_id,
            character_id,
            &character_name,
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
