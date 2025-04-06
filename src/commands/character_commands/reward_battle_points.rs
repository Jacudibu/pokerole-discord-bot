use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::character_commands::{
    build_character_list, change_character_stat, ActionType,
};
use crate::commands::{parse_variadic_args, send_error, Error};
use crate::shared::{emoji, PoiseContext};

/// Reward players with cash.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn reward_battle_points(
    ctx: PoiseContext<'_>,
    amount: i16,
    #[description = "Which character?"]
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

    match change_character_stat(
        &ctx,
        "battle_points",
        &args,
        amount as i64,
        ActionType::Reward,
    )
    .await
    {
        Ok(characters) => {
            ctx.say(format!(
                "{} received {} {}!",
                build_character_list(&characters),
                amount,
                emoji::BATTLE_POINT
            ))
            .await?;
        }
        Err(err) => {
            send_error(&ctx, err.as_str()).await?;
        }
    }

    Ok(())
}
