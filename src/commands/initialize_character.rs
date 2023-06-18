use serenity::model::user::User;
use crate::commands::{Context, Error};

/// Display item description
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
    let mut message = ctx.channel_id().send_message(ctx, |f|
        f.content("[Placeholder. This should get replaced within a couple seconds.]")
    ).await?;

    let user_id = player.id.0 as i64;
    let stat_message_id = message.id.0 as i64;
    let result = sqlx::query!(
        "INSERT INTO characters (user_id, name, stat_message_id, experience, money) VALUES (?, ?, ?, ?, ?)",
        user_id,
        name,
        stat_message_id,
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

        message.edit(ctx, |f| f.content(name)).await?;
        return Ok(());
    }

    ctx.send(|b| {
        b.content("Something went wrong!");
        b.ephemeral(true)
    }).await?;
    message.delete(ctx).await?;


    Ok(())
}
