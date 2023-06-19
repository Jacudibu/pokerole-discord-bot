use crate::emoji;
use crate::commands::{Context, Error, send_error};
use crate::commands::characters::{send_stale_data_error, update_character_post};
use crate::commands::autocompletion::autocomplete_character_name;

/// Reward players with cash.
#[poise::command(slash_command, guild_only, default_member_permissions = "ADMINISTRATOR")]
pub async fn reward_experience(
    ctx: Context<'_>,
    amount: i16,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    name: String,
) -> Result<(), Error> {
    // TODO: Button to undo the transaction which lasts for a minute or so.

    let guild_id = ctx.guild_id().expect("Command is guild_only").0 as i64;

    let record = sqlx::query!(
        "SELECT id, user_id, name, experience FROM characters WHERE name = ? AND guild_id = ?",
        name,
        guild_id
    ).fetch_one(&ctx.data().database)
        .await;

    match record {
        Ok(record) => {
            let new_value = record.experience + amount as i64;
            let result = sqlx::query!("UPDATE characters SET experience = ? WHERE id = ? AND experience = ?",
                new_value,
                record.id,
                record.experience
            ).execute(&ctx.data().database).await?;

            if result.rows_affected() != 1 {
                return send_stale_data_error(&ctx).await;
            }

            ctx.say(format!("{} received {} experience points!", record.name, amount)).await?;
            update_character_post(&ctx, record.id).await
        }
        Err(_) => {
            send_error(&ctx, format!("Unable to find a character named {}", name).as_str()).await
        },
    }
}

