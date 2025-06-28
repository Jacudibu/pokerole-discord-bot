use crate::shared::constants;
use log::error;
use serenity::all::{
    ChannelId, Context, CreateActionRow, CreateMessage, EditMessage, EditThread, HttpError, Message,
};

pub async fn handle_error_during_message_edit(
    ctx: &Context,
    e: serenity::Error,
    mut message_to_edit: Message,
    updated_message_content: impl Into<String>,
    components: Option<Vec<CreateActionRow>>,
    name: impl Into<String>,
    reply_channel_id: Option<ChannelId>,
) {
    if let serenity::Error::Http(HttpError::UnsuccessfulRequest(e)) = &e {
        if e.error.code == constants::discord_error_codes::ARCHIVED_THREAD {
            if let Ok(channel) = message_to_edit.channel(ctx).await {
                if let Some(mut channel) = channel.guild() {
                    match channel
                        .edit_thread(ctx, EditThread::new().archived(false))
                        .await
                    {
                        Ok(_) => {
                            let mut edit_message =
                                EditMessage::new().content(updated_message_content);
                            if let Some(components) = components {
                                edit_message = edit_message.components(components);
                            }

                            if let Err(e) = message_to_edit.edit(ctx, edit_message).await {
                                let _ = log_error(ctx, format!(
                                    "**Failed to update the stat message for {}!**.\nThe change has been tracked, but whilst updating the message some error occurred:\n```{:?}```\n",
                                    name.into(),
                                    e,
                                )).await;
                            }
                        }
                        Err(e) => {
                            let name = name.into();
                            log_error(ctx, format!(
                                "Some very random error occurred when updating the stat message for {}.\n**The requested change has been applied, but it isn't shown in the message there right now.**\n Error:\n```{:?}```",
                                &name, e)
                            ).await;
                            if let Some(reply_channel_id) = reply_channel_id {
                                let _ = reply_channel_id.say(ctx, &format!(
                                    "Some very random error occurred when updating the stat message for {}.\n**The requested change has been applied, but it isn't shown in the message there right now.**\n*This has been logged.*",
                                    &name)).await;
                            }
                        }
                    }

                    return;
                }
            }
        }
    }

    let name = name.into();
    log_error(ctx, format!(
        "Some very random error occurred when updating the stat message for {}.\n**The requested change has been applied, but it isn't shown in the message there right now.**\n Error:\n```{:?}```",
        &name, e)).await;
    if let Some(reply_channel_id) = reply_channel_id {
        let _ = reply_channel_id.say(ctx, &format!(
            "Some very random error occurred when updating the stat message for {}.\n**The requested change has been applied, but it isn't shown in the message there right now.**\n*This has been logged.*",
            &name)).await;
    }
}

pub async fn log_error(context: &Context, text: impl Into<String>) {
    let string = text.into();
    error!("{}", string);
    let _ = constants::ERROR_LOG_CHANNEL
        .send_message(context, CreateMessage::new().content(string))
        .await;
}
