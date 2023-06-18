use serenity::model::user::User;
use crate::commands::{Context, Error, send_error};
use crate::commands::characters::update_character_post;

/// Create a new character within the database.
#[poise::command(slash_command)]
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
    let stat_message_id = message.id.0 as i64;
    let stat_channel_id = message.channel_id.0 as i64;

    // TODO: Unique constraints
    // TODO: also track guild_id to allow multi-server stuff

    let result = sqlx::query!(
        "INSERT INTO characters (user_id, name, stat_message_id, stat_channel_id, experience, money) VALUES (?, ?, ?, ?, ?, ?)",
        user_id,
        name,
        stat_message_id,
        stat_channel_id,
        exp,
        money
    ).execute(&ctx.data().database)
        .await;

    if let Ok(bla) = result {
        if bla.rows_affected() == 1 {
            ctx.send(|b| b
                .content("Character has been successfully created!")
                .ephemeral(true)
            ).await?;
        }

        update_character_post(&ctx, user_id, name).await?;
        return Ok(());
    }

    send_error(&ctx, "Something went wrong!").await?;
    message.delete(ctx).await?;

    Ok(())
}
