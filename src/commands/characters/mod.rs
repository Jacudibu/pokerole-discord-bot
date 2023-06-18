use crate::commands::Context;
use crate::Error;

pub mod initialize_character;
pub mod reward_money;

pub async fn update_character_post<'a>(ctx: &Context<'a>, user_id: i64, name: String) -> Result<(), Error> {
    if let Some(result) = build_character_string(ctx, user_id, name).await {
        let message = ctx.serenity_context().http.get_message(result.1 as u64, result.2 as u64).await;
        if let Ok(mut message) = message {
            message.edit(ctx, |f| f.content(result.0)).await?;
        }
    }

    Ok(())
}

// TODO: we really should just change this to a query_as thingy...
pub async fn build_character_string<'a>(ctx: &Context<'a>, user_id: i64, name: String) -> Option<(String, i64, i64)> {
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
        .await;

    match entry {
        Ok(entry) => {
            let level = entry.experience / 100 + 1;
            let experience = entry.experience % 100;
            // TODO: Rank Emoji

            Some((format!("\
### Stats for {}
**Level**: {} `({} / 100)`
**Poke**: {}",
                         name, level, experience, entry.money), entry.stat_channel_id, entry.stat_message_id))
        }
        Err(_) => None,
    }
}
