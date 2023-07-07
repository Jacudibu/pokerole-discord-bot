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
