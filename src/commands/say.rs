use crate::commands::{Context, Error};
use crate::helpers;

/// Make the k4rpOS say something.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn say(ctx: Context<'_>, text: String) -> Result<(), Error> {
    for text in helpers::split_long_messages_with_custom_max_length(
        text,
        helpers::DISCORD_MESSAGE_LENGTH_LIMIT - 2 * 4,
    ) {
        ctx.say(format!("```[{}]```", text)).await?;
    }
    Ok(())
}
