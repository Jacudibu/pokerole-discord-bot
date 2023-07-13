use crate::cache::CharacterCacheItem;
use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::characters::update_character_post;
use crate::commands::{
    parse_user_input_to_character, send_ephemeral_reply, send_error, Context, Error,
};

async fn post_character_post<'a>(
    ctx: &Context<'a>,
    character: CharacterCacheItem,
) -> Result<(), Error> {
    let message = ctx
        .channel_id()
        .send_message(ctx, |f| {
            f.content("[Placeholder. This should get replaced or deleted within a couple seconds.]")
        })
        .await?;

    let stat_message_id = message.id.0 as i64;
    let stat_channel_id = message.channel_id.0 as i64;

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
            update_character_post(ctx, character.id).await?;
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
    let guild_id = ctx.guild_id().expect("Command is guild_only").0;
    let character_option = parse_user_input_to_character(&ctx, guild_id, &name).await;

    if let Some(character) = character_option {
        post_character_post(&ctx, character).await
    } else {
        send_error(&ctx, &format!("Unable to find a character named {}.", name)).await
    }
}
