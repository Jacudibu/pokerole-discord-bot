use crate::commands::{Context, Error, send_ephemeral_reply, send_error};
use crate::commands::characters::{update_character_post, validate_user_input};
use crate::commands::autocompletion::autocomplete_character_name;

/// Posts a new character stat post in case the old one got lost or deleted.
#[poise::command(slash_command, guild_only, default_member_permissions = "ADMINISTRATOR")]
pub async fn initialize_character_post(
    ctx: Context<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    name: String,
) -> Result<(), Error> {
    if let Err(e) = validate_user_input(name.as_str()) {
        return send_error(&ctx, e).await;
    }

    let message = ctx.channel_id().send_message(ctx, |f|
        f.content("[Placeholder. This should get replaced or deleted within a couple seconds.]")
    ).await?;

    let guild_id = ctx.guild_id().expect("Command is guild_only").0 as i64;
    let stat_message_id = message.id.0 as i64;
    let stat_channel_id = message.channel_id.0 as i64;

    let record = sqlx::query!(
        "UPDATE character SET stat_message_id = ?, stat_channel_id = ? WHERE guild_id = ? AND name = ? RETURNING id",
        stat_message_id,
        stat_channel_id,
        guild_id,
        name,
    ).fetch_one(&ctx.data().database)
        .await;

    if let Ok(record) = record {
        send_ephemeral_reply(&ctx, "Post has been created!").await?;
        update_character_post(&ctx, record.id.unwrap()).await?;
        ctx.data().cache.update_character_names(&ctx.data().database).await;
        return Ok(());
    }

    send_error(&ctx, "Something went wrong! Does a character with this name exist on this server for this specific player?").await?;
    message.delete(ctx).await?;

    Ok(())
}
