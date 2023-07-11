use crate::data::Data;
use crate::enums::QuestParticipantSelectionMechanism;
use crate::{helpers, Error};
use serenity::client::Context;

pub mod quest_add_random_participants;
pub mod quest_sign_out;
pub mod quest_sign_up;

async fn update_quest_message(
    context: &Context,
    data: &Data,
    channel_id: i64,
) -> Result<(), Error> {
    let quest_record = sqlx::query!(
        "SELECT bot_message_id, maximum_participant_count, participant_selection_mechanism FROM quest WHERE channel_id = ?",
        channel_id
    )
        .fetch_one(&data.database)
        .await?;

    let selection_mechanism =
        QuestParticipantSelectionMechanism::from_repr(quest_record.participant_selection_mechanism)
            .expect("Should always be valid!");

    let text = helpers::generate_quest_post_message_content(
        data,
        channel_id,
        quest_record.maximum_participant_count,
        selection_mechanism,
    )
    .await?;

    let message = context
        .http
        .get_message(channel_id as u64, quest_record.bot_message_id as u64)
        .await;
    if let Ok(mut message) = message {
        message
            .edit(context, |edit| {
                edit.content(text).components(|components| {
                    helpers::create_quest_signup_buttons(components, selection_mechanism)
                })
            })
            .await?;
    }
    Ok(())
}
