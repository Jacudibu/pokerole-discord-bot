use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::{Error, find_character};
use crate::shared::action_log::LogActionArguments;
use crate::shared::{PoiseContext, retire_character};
use serenity::all::UserId;

/// Removes a character from this guilds roster.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn retire_character(
    ctx: PoiseContext<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    character: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;

    let _ = ctx.defer().await;

    let message = retire_character::retire_character_with_id(
        ctx.serenity_context(),
        ctx.data(),
        LogActionArguments::triggered_by_user(&ctx),
        character.id,
        UserId::new(character.user_id),
        &character.name,
    )
    .await?;

    let _ = ctx.reply(message).await;

    Ok(())
}
