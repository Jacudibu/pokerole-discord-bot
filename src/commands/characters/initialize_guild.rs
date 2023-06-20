use serenity::model::channel::Channel;
use crate::commands::{Context, Error, send_ephemeral_reply, send_error};
use crate::commands::characters::{ActionType, log_action};

/// Create a new guild within the database.
#[poise::command(slash_command, guild_only, default_member_permissions = "ADMINISTRATOR")]
pub async fn initialize_guild(
    ctx: Context<'_>,
    action_log_channel: Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").0 as i64;
    let action_log_channel_id = action_log_channel.id().0 as i64;

    let record = sqlx::query!(
        "INSERT INTO guild (id, action_log_channel_id, money) VALUES (?, ?, ?)",
        guild_id,
        action_log_channel_id,
        0
    ).execute(&ctx.data().database)
        .await;

    if record.is_ok() {
        send_ephemeral_reply(&ctx, "Guild has been successfully initialized!").await?;
        log_action(ActionType::Initialization, &ctx, "Guild has been initialized. I recommend muting this channel, lul.").await?;
        return Ok(());
    }

    send_error(&ctx, "Something went wrong! Has this guild already been initialized?").await
}
