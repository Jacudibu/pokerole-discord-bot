use crate::commands::{Context, Error, send_error};
use crate::commands::characters::{log_action, send_stale_data_error, update_character_post};
use crate::commands::autocompletion::autocomplete_character_name;

/// Reward players with cash.
#[poise::command(slash_command, guild_only, default_member_permissions = "ADMINISTRATOR")]
pub async fn complete_quest(
    ctx: Context<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    name: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").0 as i64;

    let record = sqlx::query!(
        "SELECT id, user_id, name, completed_quest_count FROM character WHERE name = ? AND guild_id = ?",
        name,
        guild_id
    ).fetch_one(&ctx.data().database)
        .await;

    match record {
        Ok(record) => {
            let new_value = record.completed_quest_count + 1;
            let result = sqlx::query!("UPDATE character SET completed_quest_count = ? WHERE id = ? AND completed_quest_count = ?",
                new_value,
                record.id,
                record.completed_quest_count
            ).execute(&ctx.data().database).await?;

            if result.rows_affected() != 1 {
                return send_stale_data_error(&ctx).await;
            }

            ctx.say(format!("{} completed a quest!", record.name)).await?;
            update_character_post(&ctx, record.id).await?;
            log_action(&ctx, format!("{} added a quest completion for {}", ctx.author().name, record.name).as_str()).await
        }
        Err(_) => {
            send_error(&ctx, format!("Unable to find a character named {}", name).as_str()).await
        },
    }
}

