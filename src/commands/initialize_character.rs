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

    ctx.send(|b| {
        b.content("Something went wrong!");
        b.ephemeral(true)
    }).await?;
    message.delete(ctx).await?;

    Ok(())
}

pub async fn update_character_post<'a>(ctx: &Context<'a>, user_id: i64, name: String) -> Result<(), Error> {
    // "SELECT" will return to "entry" the rowid of the todo rows where the user_Id column = user_id.
    let entry = sqlx::query!(
                "SELECT experience, money, stat_message_id, stat_channel_id \
                FROM characters WHERE user_id = ? AND name = ? \
                ORDER BY rowid \
                LIMIT 1",
                user_id,
                name,
            )
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    // TODO: Handle error in case message wasn't found

    let message = ctx.serenity_context().http.get_message(entry.stat_channel_id as u64, entry.stat_message_id as u64).await;
    if let Ok(mut message) = message {
        message.edit(ctx, |f| f.content(format!("**{}**\n**Experience**: {}\n**Money**: {}", name, entry.experience, entry.money))).await?;
    }

    Ok(())
}
