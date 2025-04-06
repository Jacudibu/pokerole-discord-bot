use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::{parse_character_names, parse_variadic_args, send_error, Error};
use crate::shared::helpers;
use crate::shared::PoiseContext;
use chrono::Utc;

/// Manually add a character to the quest, skipping any waiting queue.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn add_quest_participant(
    ctx: PoiseContext<'_>,
    #[autocomplete = "autocomplete_character_name"] character1: String,
    #[autocomplete = "autocomplete_character_name"] character2: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character3: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character4: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character5: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character6: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character7: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character8: Option<String>,
    #[autocomplete = "autocomplete_character_name"] character9: Option<String>,
) -> Result<(), Error> {
    let args = parse_variadic_args(
        character1, character2, character3, character4, character5, character6, character7,
        character8, character9,
    );

    let channel_id = ctx.channel_id().get() as i64;
    let guild_id = ctx.guild_id().expect("Command is guild_only!");

    let existing_quest = sqlx::query!("SELECT * FROM quest WHERE channel_id = ?", channel_id)
        .fetch_optional(&ctx.data().database)
        .await?;

    if existing_quest.is_none() {
        return send_error(
            &ctx,
            "Doesn't look like there was a quest created within this channel!",
        )
        .await;
    }

    let characters = parse_character_names(&ctx, guild_id.get(), &args).await?;

    let timestamp = Utc::now().timestamp();
    let mut result =
        String::from("Manually signed up the following character(s) for this quest:\n");
    for x in characters {
        sqlx::query!("INSERT INTO quest_signup (quest_id, character_id, timestamp, accepted) VALUES (?, ?, ?, ?)
ON CONFLICT (quest_id, character_id) DO UPDATE SET accepted = true", 
            channel_id,
            x.id,
            timestamp,
            true,
        ).execute(&ctx.data().database).await?;
        result.push_str(format!("- {} (<@{}>)\n", x.name, x.user_id).as_str())
    }

    ctx.say(result).await?;
    helpers::update_quest_message(ctx.serenity_context(), ctx.data(), channel_id).await?;
    Ok(())
}
