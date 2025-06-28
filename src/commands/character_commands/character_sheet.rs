use serenity::all::ChannelId;

use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::{find_character, send_ephemeral_reply, Error};
use crate::shared::errors::DatabaseError;
use crate::shared::utility::channel_id_ext::ChannelIdExt;
use crate::shared::PoiseContext;

/// Pulls up a link to a character sheet
#[poise::command(slash_command, guild_only)]
pub async fn character_sheet(
    ctx: PoiseContext<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    character: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;

    match sqlx::query!(
        "SELECT stat_channel_id FROM character WHERE id = ?",
        character.id
    )
    .fetch_one(&ctx.data().database)
    .await
    {
        Ok(record) => {
            let channel_id = ChannelId::new(record.stat_channel_id as u64);
            let _ = send_ephemeral_reply(&ctx, channel_id.channel_id_link()).await;
        }
        Err(e) => {
            return Err(Box::new(DatabaseError::new(&format!(
                "Encountered an error when looking up character {} (id {}) in db: {}",
                character.name,
                character.id,
                e.to_string()
            ))));
        }
    }

    Ok(())
}
