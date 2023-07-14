use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::characters::{build_character_list, update_character_post};
use crate::commands::{parse_character_names, parse_variadic_args, send_error, Context, Error};
use chrono::Utc;

/// Use this to increase the quest completion counter.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn complete_quest(
    ctx: Context<'_>,
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
    let args = parse_variadic_args(
        character1, character2, character3, character4, character5, character6, character7,
        character8, character9,
    );

    let channel_id = ctx.channel_id().0 as i64;
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

    let characters = parse_character_names(&ctx, guild_id.0, &args).await?;
    let timestamp = Utc::now().timestamp();
    sqlx::query!(
        "UPDATE quest SET completion_timestamp = ? WHERE channel_id = ?",
        timestamp,
        channel_id
    )
    .execute(&ctx.data().database)
    .await?;

    for x in &characters {
        sqlx::query!(
            "INSERT INTO quest_completion (quest_id, character_id) VALUES (?, ?) ON CONFLICT DO NOTHING",
            channel_id,
            x.id,
        )
        .execute(&ctx.data().database)
        .await?;
    }

    ctx.say(format!(
        "{} completed a quest!",
        build_character_list(&characters)
    ))
    .await?;

    for x in characters {
        update_character_post(&ctx, x.id).await?;
    }

    Ok(())
}
