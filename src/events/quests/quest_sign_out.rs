use crate::data::Data;
use crate::{helpers, Error};
use serenity::client::Context;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::model::prelude::InteractionResponseType;

pub async fn quest_sign_out(
    context: &Context,
    interaction: &MessageComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let guild_id = interaction
        .guild_id
        .expect("Command should be guild_only")
        .0 as i64;
    let user_id = interaction.user.id.0 as i64;
    let channel_id = interaction.channel_id.0 as i64;

    let rows_affected = execute_sign_out(data, guild_id, user_id, channel_id).await?;

    let text = if rows_affected > 0 {
        "Removed your signups!"
    } else {
        "Seems like you weren't signed up for this quest in the first place!"
    };

    interaction
        .create_interaction_response(context, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data.ephemeral(true)
                        .content(text)
                        .components(|components| components)
                })
        })
        .await?;

    if rows_affected > 0 {
        helpers::update_quest_message(context, data, channel_id).await?;
    }

    Ok(())
}

async fn execute_sign_out(
    data: &Data,
    guild_id: i64,
    user_id: i64,
    channel_id: i64,
) -> Result<u64, Error> {
    let result = sqlx::query!(
        "DELETE FROM quest_signup WHERE quest_id = ? AND character_id IN (SELECT id as character_id FROM character WHERE user_id = ? AND guild_id = ?)",
        channel_id,
        user_id,
        guild_id
    )
        .execute(&data.database)
        .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use crate::enums::QuestParticipantSelectionMechanism;
    use crate::events::quests::quest_sign_out::execute_sign_out;
    use crate::{database_helpers, Error};
    use sqlx::{Pool, Sqlite};

    #[sqlx::test]
    async fn sign_out(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let channel_id = 100;
        let user1_id = 200;
        let user2_id = 201;
        let guild_id = 300;
        let bot_message_id = 400;
        let character11_id = 511;
        let character12_id = 512;
        let character21_id = 521;
        let character11_name = String::from("test11");
        let character12_name = String::from("test12");
        let character21_name = String::from("test21");

        database_helpers::create_mock::guild(&data.database, guild_id).await;
        database_helpers::create_mock::user(&data.database, user1_id).await;
        database_helpers::create_mock::user(&data.database, user2_id).await;
        database_helpers::create_mock::quest(
            &data.database,
            channel_id,
            guild_id,
            user1_id,
            bot_message_id,
            5,
            QuestParticipantSelectionMechanism::Random,
        )
        .await;
        database_helpers::create_mock::character(
            &data,
            guild_id,
            user1_id,
            character11_id,
            &character11_name,
        )
        .await;
        database_helpers::create_mock::character(
            &data,
            guild_id,
            user1_id,
            character12_id,
            &character12_name,
        )
        .await;
        database_helpers::create_mock::character(
            &data,
            guild_id,
            user2_id,
            character21_id,
            &character21_name,
        )
        .await;
        database_helpers::create_mock::quest_signup(&data.database, channel_id, character11_id)
            .await;
        database_helpers::create_mock::quest_signup(&data.database, channel_id, character12_id)
            .await;
        database_helpers::create_mock::quest_signup(&data.database, channel_id, character21_id)
            .await;

        execute_sign_out(&data, guild_id, user1_id, channel_id).await?;

        let signups = sqlx::query!("SELECT quest_id, character_id, timestamp FROM quest_signup")
            .fetch_all(&data.database)
            .await?;

        assert_eq!(1, signups.len());
        let signup = signups.first().unwrap();
        assert_eq!(channel_id, signup.quest_id);
        assert_eq!(character21_id, signup.character_id);

        Ok(())
    }
}
