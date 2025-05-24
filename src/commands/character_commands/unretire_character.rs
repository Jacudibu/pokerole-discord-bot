use crate::commands::autocompletion::autocomplete_retired_character_name;
use crate::commands::character_commands::{log_action, ActionType};
use crate::commands::{find_character, update_character_post, Error};
use crate::shared::{constants, PoiseContext};

/// Unretire a character.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn unretire_character(
    ctx: PoiseContext<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_retired_character_name"]
    character: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;

    match sqlx::query!(
        "UPDATE character SET is_retired = false WHERE id = ?",
        character.id
    )
    .execute(&ctx.data().database)
    .await
    {
        Ok(_) => {
            let _ = ctx
                .reply(&format!(
                    "{} has returned from their retirement.",
                    character.name
                ))
                .await;

            let _ = log_action(
                &ActionType::CharacterUnRetirement,
                &ctx,
                &format!("{} has returned from their retirement.", character.name),
            )
            .await;

            ctx.data()
                .cache
                .update_character_names(&ctx.data().database)
                .await;
            update_character_post(&ctx, character.id).await;
        }
        Err(e) => {
            let _ = ctx
                .reply(&format!(
                    "Something went wrong when trying to un-retire {}:\n```{:?}```\n{}, please look into this.",
                    character.name,
                    e,
                    constants::ADMIN_PING_STRING
                ))
                .await;
        }
    }

    Ok(())
}
