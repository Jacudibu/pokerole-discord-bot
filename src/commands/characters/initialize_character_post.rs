use crate::cache::CharacterCacheItem;
use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::characters::update_character_post;
use crate::commands::{find_character, send_ephemeral_reply, send_error, Context, Error};
use serenity::all::CreateMessage;

async fn post_character_post<'a>(
    ctx: &Context<'a>,
    character: CharacterCacheItem,
) -> Result<(), Error> {
    let message = ctx
        .channel_id()
        .send_message(
            ctx,
            CreateMessage::new().content(
                "[Placeholder. This should get replaced or deleted within a couple seconds.]",
            ),
        )
        .await?;

    let stat_message_id = message.id.get() as i64;
    let stat_channel_id = message.channel_id.get() as i64;

    let record = sqlx::query!(
        "UPDATE character SET stat_message_id = ?, stat_channel_id = ? WHERE id = ?",
        stat_message_id,
        stat_channel_id,
        character.id
    )
    .execute(&ctx.data().database)
    .await;

    if let Ok(record) = record {
        if record.rows_affected() == 1 {
            send_ephemeral_reply(ctx, "Post has been created!").await?;
            update_character_post(ctx, character.id).await;
            ctx.data()
                .cache
                .update_character_names(&ctx.data().database)
                .await;
            return Ok(());
        }
    }

    send_error(ctx, "Something went wrong! Does a character with this name exist on this server for this specific player?").await?;
    message.delete(ctx).await?;

    Ok(())
}

/// Posts a new character stat post in case the old one got lost or deleted.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn initialize_character_post(
    ctx: Context<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    name: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &name).await?;

    post_character_post(&ctx, character).await
}
