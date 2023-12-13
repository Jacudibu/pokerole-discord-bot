use crate::commands::characters::{log_action, ActionType};
use crate::commands::{send_ephemeral_reply, send_error, Context, Error};
use serenity::model::channel::Channel;

/// Create a new guild within the database.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn initialize_guild(ctx: Context<'_>, action_log_channel: Channel) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get() as i64;
    let action_log_channel_id = action_log_channel.id().get() as i64;

    let record = sqlx::query!(
"INSERT INTO guild (id, action_log_channel_id) VALUES (?, ?)
ON CONFLICT (id) DO UPDATE SET action_log_channel_id = excluded.action_log_channel_id WHERE id = excluded.id",
        guild_id,
        action_log_channel_id,
    ).execute(&ctx.data().database)
        .await;

    if record.is_ok() {
        send_ephemeral_reply(&ctx, "Guild has been successfully initialized!").await?;
        log_action(&ActionType::Initialization, &ctx, "The action log channel has been set to this lovely place here. I recommend muting this channel, lul.").await?;
        Ok(())
    } else {
        send_error(
            &ctx,
            "Something went wrong! Has this guild already been initialized?",
        )
        .await
    }
}
