use crate::commands::Error;
use crate::shared::constants;
use crate::shared::utility::message_splitting;
use crate::shared::PoiseContext;

/// Make the k4rpOS say something.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn say(ctx: PoiseContext<'_>, text: String) -> Result<(), Error> {
    for text in message_splitting::split_long_messages_with_custom_max_length(
        text,
        constants::DISCORD_MESSAGE_LENGTH_LIMIT - 2 * 4,
    ) {
        ctx.say(format!("```[{}]```", text)).await?;
    }
    Ok(())
}
