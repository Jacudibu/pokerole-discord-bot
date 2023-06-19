use crate::commands::Context;
use crate::Error;

pub mod initialize_character;
pub mod reward_money;

pub async fn update_character_post<'a>(ctx: &Context<'a>, id: i64) -> Result<(), Error> {
    if let Some(result) = build_character_string(ctx, id).await {
        let message = ctx.serenity_context().http.get_message(result.1 as u64, result.2 as u64).await;
        if let Ok(mut message) = message {
            message.edit(ctx, |f| f.content(result.0)).await?;
        }
    }

    Ok(())
}

// TODO: we really should just change this to a query_as thingy...
pub async fn build_character_string<'a>(ctx: &Context<'a>, character_id: i64) -> Option<(String, i64, i64)> {
    let entry = sqlx::query!(
                "SELECT name, experience, money, stat_message_id, stat_channel_id \
                FROM characters WHERE id = ? \
                ORDER BY rowid \
                LIMIT 1",
                character_id,
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
                         entry.name, level, experience, entry.money), entry.stat_channel_id, entry.stat_message_id))
        }
        Err(_) => None,
    }
}
