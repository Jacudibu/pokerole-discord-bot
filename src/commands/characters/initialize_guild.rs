use serenity::model::channel::Channel;
use crate::commands::{Context, Error, send_ephemeral_reply, send_error};

/// Create a new guild within the database.
#[poise::command(slash_command, guild_only, default_member_permissions = "ADMINISTRATOR")]
pub async fn initialize_guild(
    ctx: Context<'_>,
    transaction_channel: Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").0 as i64;
    let transaction_channel_id = transaction_channel.id().0 as i64;

    let record = sqlx::query!(
        "INSERT INTO guild (id, transaction_channel_id) VALUES (?, ?)",
        guild_id,
        transaction_channel_id
    ).fetch_one(&ctx.data().database)
        .await;

    if let Ok(record) = record {
        send_ephemeral_reply(&ctx, "Guild has been successfully initialized!").await?;
        transaction_channel.id().send_message(ctx, |f| f.content("Guild has been initialized. I recommend muting this channel, lul."));
        return Ok(());
    }

    send_error(&ctx, "Something went wrong! Has this guild already been initialized?").await
}
