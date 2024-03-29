use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::characters::{log_action, ActionType};
use crate::commands::{find_character, update_character_post, Context, Error};
use crate::errors::CommandInvocationError;
use crate::helpers;
use serenity::all::{ChannelId, EditThread};
use tokio::join;

/// Removes a character from this guilds roster.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn retire_character(
    ctx: Context<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    character: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;

    match sqlx::query!(
        "UPDATE character SET is_retired = true WHERE id = ?",
        character.id
    )
    .execute(&ctx.data().database)
    .await
    {
        Ok(_) => {
            let message = format!("{} has been retired.", character.name);
            let a = ctx.reply(&message);
            let b = log_action(&ActionType::CharacterRetirement, &ctx, &message);
            let c = ctx.data().cache.reset(&ctx.data().database);
            let d = update_character_post(&ctx, character.id);

            let (_, _, _, _) = join!(a, b, c, d);

            archive_character_post(&ctx, character.id).await;
        }
        Err(e) => {
            return Err(Box::new(
                CommandInvocationError::new(&format!(
                    "Something went wrong when trying to retire {}:\n```{:?}```",
                    character.name, e,
                ))
                .log(),
            ));
        }
    }

    Ok(())
}

async fn archive_character_post(ctx: &Context<'_>, character_id: i64) {
    if let Ok(result) = sqlx::query!(
        "SELECT stat_channel_id FROM character WHERE id = ?",
        character_id
    )
    .fetch_one(&ctx.data().database)
    .await
    {
        let channel_id = ChannelId::new(result.stat_channel_id as u64);
        let _ = channel_id
            .edit_thread(&ctx, EditThread::new().archived(true))
            .await;
    }
}
