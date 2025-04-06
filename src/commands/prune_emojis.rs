use crate::commands::send_ephemeral_reply;
use crate::shared::{helpers, PoiseContext};
use crate::Error;

/// Removes emoji which have been manually deleted from the server from the database.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn prune_emojis(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only!");
    let emojis = ctx.serenity_context().http.get_emojis(guild_id).await?;
    let guild_id = guild_id.get() as i64;

    let db_emojis = sqlx::query!(
        "SELECT discord_string FROM emoji WHERE guild_id = ?",
        guild_id
    )
    .fetch_all(&ctx.data().database)
    .await?;

    let mut list = Vec::new();
    for record in db_emojis {
        if !emojis
            .iter()
            .any(|emoji| emoji.to_string() == record.discord_string)
        {
            sqlx::query!(
                "DELETE FROM emoji WHERE discord_string = ?",
                record.discord_string
            )
            .execute(&ctx.data().database)
            .await?;

            list.push(record.discord_string);
        }
    }

    if list.is_empty() {
        send_ephemeral_reply(&ctx, "Didn't find any deleted emojis!").await?;
        return Ok(());
    }

    let text = format!("Removed {} emojis.\n```{}```", list.len(), list.join("\n"));
    for text in helpers::split_long_messages(text) {
        send_ephemeral_reply(&ctx, &text).await?;
    }

    Ok(())
}
