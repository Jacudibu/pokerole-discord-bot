use crate::data::Data;
use crate::{helpers, Error};
use rand::Rng;
use serenity::all::{
    ComponentInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::client::Context;

pub async fn quest_add_random_participants(
    context: &Context,
    interaction: &ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let user_id = interaction.user.id.get() as i64;
    let channel_id = interaction.channel_id.get() as i64;

    let execution_result = execute(data, user_id, channel_id).await;
    let was_error = execution_result.is_err();
    let result = execution_result.unwrap_or_else(|error| error.to_string());

    interaction
        .create_response(
            context,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(was_error)
                    .content(result),
            ),
        )
        .await?;

    if !was_error {
        helpers::update_quest_message(context, data, channel_id).await?;
    }

    Ok(())
}

async fn execute(data: &Data, user_id: i64, channel_id: i64) -> Result<String, Error> {
    let quest_record = sqlx::query!(
        "SELECT creator_id, maximum_participant_count FROM quest WHERE channel_id = ?",
        channel_id
    )
    .fetch_one(&data.database)
    .await?;

    if quest_record.creator_id != user_id {
        return Err(Error::from("Only the quest owner can do this, sorry!"));
    }

    let accepted_participants = sqlx::query!(
        "SELECT COUNT(*) as count FROM quest_signup WHERE quest_id = ? AND accepted = true",
        channel_id
    )
    .fetch_one(&data.database)
    .await?;

    if accepted_participants.count as i64 >= quest_record.maximum_participant_count {
        return Err(Error::from("The quest is already full! If you want to add more participants, either add them manually or remove one of the already accepted players."));
    }

    let mut floating_participants = sqlx::query!(
        "SELECT character.id as character_id, character.name as character_name, character.user_id as user_id
FROM quest_signup
INNER JOIN character ON
    quest_signup.character_id = character.id
WHERE quest_signup.quest_id = ? AND quest_signup.accepted = false
ORDER BY quest_signup.timestamp DESC",
        channel_id
    )
    .fetch_all(&data.database)
    .await?;

    if floating_participants.is_empty() {
        return Err(Error::from(
            "Doesn't seem like there are any participants waiting for a spot!",
        ));
    }

    let mut result = String::from("The following participants where randomly chosen:\n");
    let mut chosen_character_ids = Vec::new();
    {
        let mut rng = rand::thread_rng();
        for _ in 0..quest_record.maximum_participant_count - accepted_participants.count as i64 {
            if floating_participants.is_empty() {
                break;
            }

            let index = rng.gen_range(0..floating_participants.len());
            let winner = floating_participants.remove(index);

            chosen_character_ids.push(winner.character_id);
            result.push_str(
                format!("- {} (<@{}>)\n", winner.character_name, winner.user_id).as_str(),
            );
        }
    }
    for x in chosen_character_ids {
        sqlx::query!(
            "UPDATE quest_signup SET accepted = true WHERE quest_id = ? AND character_id = ?",
            channel_id,
            x
        )
        .execute(&data.database)
        .await?;
    }

    Ok(result)
}
