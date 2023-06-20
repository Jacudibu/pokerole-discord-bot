use crate::commands::{Context, Error, send_error};
use crate::commands::characters::{ActionType, change_character_stat, validate_user_input};
use crate::commands::autocompletion::autocomplete_character_name;

/// Use this to increase the quest completion counter.
#[poise::command(slash_command, guild_only, default_member_permissions = "ADMINISTRATOR")]
pub async fn complete_quest(
    ctx: Context<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    name: String,
) -> Result<(), Error> {
    if let Err(e) = validate_user_input(name.as_str()) {
        return send_error(&ctx, e).await;
    }

    if let Ok(result) = change_character_stat(&ctx, "completed_quest_count", &name, 1, ActionType::Reward).await {
        ctx.say(format!("{} completed a quest!", name)).await?;
    }

    Ok(())
}
