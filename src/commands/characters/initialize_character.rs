use serenity::model::user::User;
use crate::commands::{Context, Error, send_ephemeral_reply, send_error};
use crate::commands::characters::update_character_post;

/// Create a new character within the database.
#[poise::command(slash_command, guild_only, default_member_permissions = "ADMINISTRATOR")]
pub async fn initialize_character(
    ctx: Context<'_>,
    #[description = "Who owns the character?"]
    player: User,
    #[description = "What's the character's name?"]
    name: String,
    #[min = 0_i64]
    exp: i64,
    #[min = 0_i64]
    money: i64,
) -> Result<(), Error> {
    let message = ctx.channel_id().send_message(ctx, |f|
        f.content("[Placeholder. This should get replaced or deleted within a couple seconds.]")
    ).await?;

    let user_id = player.id.0 as i64;
    let guild_id = ctx.guild_id().expect("Command is guild_only").0 as i64;
    let stat_message_id = message.id.0 as i64;
    let stat_channel_id = message.channel_id.0 as i64;

    let record = sqlx::query!(
        "INSERT INTO character (user_id, guild_id, name, stat_message_id, stat_channel_id, experience, money) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
        user_id,
        guild_id,
        name,
        stat_message_id,
        stat_channel_id,
        exp,
        money
    ).fetch_one(&ctx.data().database)
        .await;

    if let Ok(record) = record {
        send_ephemeral_reply(&ctx, "Character has been successfully created!").await?;
        update_character_post(&ctx, record.id).await?;
        ctx.data().cache.update_character_names(&ctx.data().database).await;
        return Ok(());
    }

    send_error(&ctx, "Something went wrong! Does a character with this name already exist for this specific player?").await?;
    message.delete(ctx).await?;

    Ok(())
}
