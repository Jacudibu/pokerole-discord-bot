use crate::commands::{Context, Error, send_error};
use crate::commands::characters::update_character_post;

/// Reward players with cash.
#[poise::command(slash_command)]
pub async fn reward_money(
    ctx: Context<'_>,
    amount: i16,
    #[description = "Which character?"]
    name: String,
) -> Result<(), Error> {
    // TODO: Make this exclusive to Admins and GMs
    // TODO: Autocomplete. Name should also include owner name, in order to reduce the possibility of slip ups.
    // TODO: guild_id
    // TODO: Option to also add the untaxed amount to guild stash.
    // TODO: Button to undo the transaction which lasts for a minute or so.

    let record = sqlx::query!(
        "SELECT user_id, money FROM characters WHERE name = ?",
        name,
    ).fetch_one(&ctx.data().database)
        .await;

    match record {
        Ok(record) => {
            let new_value = record.money + amount as i64;
            let result = sqlx::query!("UPDATE characters SET money=? WHERE name = ? AND money = ?",
                new_value,
                name,
                record.money
            ).execute(&ctx.data().database).await?;

            if result.rows_affected() != 1 {
                return send_error(&ctx, "Something went wrong. Try again?").await;
            }

            ctx.say(format!("{} received {} Poke!", name, amount)).await?;
            update_character_post(&ctx, record.user_id, name).await
        }
        Err(_) => {
            send_error(&ctx, format!("Unable to find a character named {}", name).as_str()).await
        },
    }
}

