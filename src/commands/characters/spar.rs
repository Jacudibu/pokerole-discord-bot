use crate::cache::CharacterCacheItem;
use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::characters::{
    build_character_list, change_character_stat_after_validation, log_action, ActionType,
};
use crate::commands::{parse_character_names, parse_variadic_args, send_error, Context, Error};

/// Reward players with cash.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn spar(
    ctx: Context<'_>,
    #[description = "Which characters are sparring?"]
    #[autocomplete = "autocomplete_character_name"]
    character1: String,
    #[autocomplete = "autocomplete_character_name"] character2: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character3: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character4: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character5: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character6: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character7: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character8: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character9: Option<String>,
) -> Result<(), Error> {
    // TODO: Button to undo the transaction which lasts for a minute or so.
    let args = parse_variadic_args(
        character1, character2, character3, character4, character5, character6, character7,
        character8, character9,
    );

    match handle_sparring(&ctx, &args).await {
        Ok(result) => {
            if result.participants_who_gained_exp.len() > 0 {
                ctx.say(format!(
                    "Tracked a sparring session for {}.\n{} received {} experience points!",
                    build_character_list(&result.participants),
                    build_character_list(&result.participants_who_gained_exp),
                    result.experience_value,
                ))
                .await?;
            } else {
                ctx.say(format!(
                    "Tracked a sparring session for {}.",
                    build_character_list(&result.participants),
                ))
                .await?;
            }
        }
        Err(err) => {
            send_error(&ctx, err.as_str()).await?;
        }
    }

    Ok(())
}

struct SparringResult {
    participants: Vec<CharacterCacheItem>,
    participants_who_gained_exp: Vec<CharacterCacheItem>,
    experience_value: i64,
}

async fn handle_sparring<'a>(
    ctx: &Context<'a>,
    names: &Vec<String>,
) -> Result<SparringResult, String> {
    let guild_id = ctx
        .guild_id()
        .expect("Commands using this function are marked as guild_only")
        .get();

    let guild_id_i64 = guild_id as i64;

    let guild_settings = sqlx::query!(
        "SELECT weekly_spar_limit, weekly_spar_reward FROM guild WHERE id = ?",
        guild_id_i64
    )
    .fetch_one(&ctx.data().database)
    .await
    .expect("Every guild should have settings!");

    match parse_character_names(ctx, guild_id, names).await {
        Ok(characters) => {
            let mut result = SparringResult {
                participants: Vec::new(),
                participants_who_gained_exp: Vec::new(),
                experience_value: guild_settings.weekly_spar_reward,
            };

            for x in &characters {
                result.participants.push(x.clone());
                let current = sqlx::query!(
                    "SELECT total_spar_count, weekly_spar_count FROM character WHERE id = ?",
                    x.id
                )
                .fetch_one(&ctx.data().database)
                .await
                .expect("Character with cached ID should exist."); // TODO: Error handling

                let new_total_spar_count = current.total_spar_count + 1;
                let new_weekly_spar_count = current.weekly_spar_count + 1;

                let _ = sqlx::query!(
                    "UPDATE character SET total_spar_count = ?, weekly_spar_count = ? WHERE id = ?",
                    new_total_spar_count,
                    new_weekly_spar_count,
                    x.id
                )
                .execute(&ctx.data().database)
                .await;

                let _ = log_action(
                    &ActionType::Spar,
                    ctx,
                    &format!("Tracked a sparring session for {}!", x.name),
                )
                .await;

                if new_total_spar_count <= guild_settings.weekly_spar_limit {
                    result.participants_who_gained_exp.push(x.clone());
                    let _ = change_character_stat_after_validation(
                        ctx,
                        "experience",
                        x,
                        guild_settings.weekly_spar_reward,
                        &ActionType::Spar,
                    )
                    .await;
                }
            }
            Ok(result)
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;
    use sqlx::{Pool, Sqlite};

    #[sqlx::test]
    async fn spar_once(db: Pool<Sqlite>) -> Result<(), Error> {
        todo!();
    }

    #[sqlx::test]
    async fn spar_more_than_weekly_limit(db: Pool<Sqlite>) -> Result<(), Error> {
        todo!();
    }
}
