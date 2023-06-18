use crate::commands::Context;
use crate::Error;

pub mod initialize_character;
pub mod reward_money;

pub async fn update_character_post<'a>(ctx: &Context<'a>, user_id: i64, name: String) -> Result<(), Error> {
    // TODO: Add guild_id into the mix
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
