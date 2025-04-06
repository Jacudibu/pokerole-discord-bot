use crate::commands::{send_ephemeral_reply, Error};
use crate::shared::errors::CommandInvocationError;
use crate::shared::PoiseContext;

/// Pin or Unpin a message
#[poise::command(context_menu_command = "Pin or Unpin")]
pub async fn pin_or_unpin(
    ctx: PoiseContext<'_>,
    #[description = "Message which should be (un)pinned."]
    message: poise::serenity_prelude::Message,
) -> Result<(), Error> {
    let audit_log = format!(
        "Requested by {} (ID: {})",
        ctx.author().name,
        ctx.author().id
    );

    match if message.pinned {
        ctx.http()
            .unpin_message(message.channel_id, message.id, Some(&audit_log))
            .await
    } else {
        ctx.http()
            .pin_message(message.channel_id, message.id, Some(&audit_log))
            .await
    } {
        Ok(_) => {
            let _ = if message.pinned {
                send_ephemeral_reply(&ctx, "Message has been unpinned!").await
            } else {
                send_ephemeral_reply(&ctx, "Message has been pinned!").await
            };
            Ok(())
        }
        Err(e) => {
            let pin_or_unpin = if message.pinned { "unpin" } else { "pin" };
            Err(Box::new(CommandInvocationError::new(&format!(
                "Failed to {} message.\n```{}```",
                pin_or_unpin, e
            ))))
        }
    }
}
