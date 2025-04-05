use tokio::join;

use crate::commands::autocompletion::autocomplete_owned_character_name;
use crate::commands::characters::{change_character_stat_after_validation, log_action, ActionType};
use crate::commands::{
    ensure_user_exists, ensure_user_owns_character, find_character, Context, Error,
};
use crate::emoji::get_character_emoji;
use crate::errors::{CommandInvocationError, ValidationError};

/// Store your GM Experience after a quest.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn use_gm_experience(
    ctx: Context<'_>,
    #[min = 1_i64] amount: i64,
    #[autocomplete = "autocomplete_owned_character_name"] character: String,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let guild_id = ctx.guild().expect("Command is guild_only!").id.get() as i64;
    ensure_user_exists(&ctx, user_id, guild_id).await;
    let character = find_character(ctx.data(), guild_id as u64, &character).await?;
    ensure_user_owns_character(ctx.author(), &character)?;

    match sqlx::query!(
        "SELECT gm_experience FROM user_in_guild WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id
    )
    .fetch_one(&ctx.data().database)
    .await
    {
        Ok(record) => {
            if amount > record.gm_experience {
                return Err(Box::new(ValidationError::new(&format!(
                    "Cannot use {} GM Experience, you only own {}.",
                    amount, record.gm_experience
                ))));
            }

            let new_amount = record.gm_experience - amount;
            match sqlx::query!(
                "UPDATE user_in_guild SET gm_experience = ? WHERE user_id = ? AND guild_id = ?",
                new_amount,
                user_id,
                guild_id
            )
            .execute(&ctx.data().database)
                .await
            {
                Ok(_) => {
                    let emoji = get_character_emoji(ctx.serenity_context(), ctx.data(), character.id).await;
                    let text = format!("Used {} GM Experience on {}{}!", amount, emoji.unwrap_or(String::new()), character.name);
                    let reply = ctx.say(&text);
                    let log = log_action(&ActionType::UseGMExperience, &ctx, &text);
                    let _ = join!(reply, log);
                    // Do this afterwards to ensure the level up message is always sent second
                    let _ = change_character_stat_after_validation(&ctx, "experience", &character, amount, &ActionType::DoNotLog).await;
                }
                Err(e) => {
                    return Err(Box::new(
                        CommandInvocationError::new(&format!(
                            "Something went wrong when applying GM Experience for a user with id {} in guild with id {}!\n```{:?}```",
                            user_id, guild_id, e
                        ))
                            .log(),
                    ))
                }
            }
        }
        Err(e) => {
            return Err(Box::new(
                CommandInvocationError::new(&format!(
                "Was unable to find a user with id {} in guild with id {} in database!\n```{:?}```",
                user_id, guild_id, e
            ))
                .log(),
            ))
        }
    };

    Ok(())
}
