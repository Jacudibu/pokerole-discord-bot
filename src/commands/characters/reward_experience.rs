use crate::commands::{Context, Error};
use crate::commands::characters::{ActionType, change_character_stat};
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
    if let Ok(_) = change_character_stat(&ctx, "experience", &name, amount as i64, ActionType::Reward).await {
        ctx.say(format!("{} received {} experience points!", name, amount)).await?;
    }

    Ok(())
}

