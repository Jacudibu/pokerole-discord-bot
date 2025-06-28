use crate::Error;
use crate::shared::SerenityContext;
use crate::shared::action_log::{ActionType, LogActionArguments, log_action};
use crate::shared::character::update_character_post_with_serenity_context;
use crate::shared::data::Data;
use crate::shared::errors::CommandInvocationError;
use serenity::all::{ChannelId, EditThread, UserId};
use sqlx::{Pool, Sqlite};
use tokio::join;

/// Retires a character with the provided id.
/// It's a bit ugly since we need to collect data between
pub async fn retire_character_with_id(
    serenity_context: &SerenityContext,
    data: &Data,
    log_action_arguments: LogActionArguments<'_>,
    character_id: i64,
    character_owner_id: UserId,
    character_name: &str,
) -> Result<String, Error> {
    match sqlx::query!(
        "UPDATE character SET is_retired = true WHERE id = ?",
        character_id
    )
    .execute(&data.database)
    .await
    {
        Ok(_) => {
            let message = format!("{} has been retired.", character_name);

            let update_names = data.cache.update_character_names(&data.database);
            let update_post = update_character_post_with_serenity_context(
                serenity_context,
                log_action_arguments.guild_id,
                log_action_arguments.channel_id,
                character_owner_id,
                data,
                character_id,
            );
            let log = log_action(
                &ActionType::CharacterRetirement,
                log_action_arguments,
                &message,
            );
            let (_, _, _) = join!(log, update_names, update_post);

            archive_character_post(serenity_context, &data.database, character_id).await;
            Ok(message)
        }
        Err(e) => Err(Box::new(
            CommandInvocationError::new(format!(
                "Something went wrong when trying to retire {}:\n```{:?}```",
                character_name, e,
            ))
            .should_be_logged(),
        )),
    }
}

async fn archive_character_post(ctx: &SerenityContext, db: &Pool<Sqlite>, character_id: i64) {
    if let Ok(result) = sqlx::query!(
        "SELECT stat_channel_id FROM character WHERE id = ?",
        character_id
    )
    .fetch_one(db)
    .await
    {
        let channel_id = ChannelId::new(result.stat_channel_id as u64);
        let _ = channel_id
            .edit_thread(&ctx, EditThread::new().archived(true))
            .await;
    }
}
