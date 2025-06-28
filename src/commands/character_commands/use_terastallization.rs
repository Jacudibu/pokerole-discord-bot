use crate::commands::autocompletion::{
    autocomplete_owned_character_name, autocomplete_pokemon_type,
};
use crate::commands::{Error, ensure_user_owns_character, find_character};
use crate::shared::PoiseContext;
use crate::shared::character::update_character_post_with_poise_context;
use crate::shared::enums::PokemonTypeWithoutShadow;
use crate::shared::errors::ValidationError;

#[derive(sqlx::FromRow)]
struct TeraCharge {
    unlocked: i64,
    used: i64,
}

/// Use a Terastallization charge.
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, guild_only)]
pub async fn use_terastallization(
    ctx: PoiseContext<'_>,
    #[autocomplete = "autocomplete_pokemon_type"]
    #[description = "Which Type?"]
    tera_type: PokemonTypeWithoutShadow,
    #[description = "Which Character?"]
    #[autocomplete = "autocomplete_owned_character_name"]
    character: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;
    ensure_user_owns_character(ctx.author(), &character)?;

    let tera_used_column = tera_type.get_tera_used_column();
    let tera_unlocked_column = tera_type.get_tera_unlocked_column();

    let record = sqlx::query_as::<_, TeraCharge>(
        format!(
            "SELECT {} as unlocked, {} as used FROM character WHERE id = ?",
            tera_unlocked_column, tera_used_column
        )
        .as_str(),
    )
    .bind(character.id)
    .fetch_one(&ctx.data().database)
    .await?;

    if record.used >= record.unlocked {
        return Err(Box::new(ValidationError::new(&format!(
            "{} doesn't seem to have any {} Terastallization charges left!",
            character.name, tera_type
        ))));
    }

    sqlx::query(&format!(
        "UPDATE character SET {} = ? WHERE id = ?",
        tera_used_column
    ))
    .bind(record.used + 1)
    .bind(character.id)
    .execute(&ctx.data().database)
    .await?;

    ctx.say(format!(
        "{} used a {} Terastallization!",
        character.name, tera_type
    ))
    .await?;

    update_character_post_with_poise_context(&ctx, character.id).await;

    Ok(())
}
