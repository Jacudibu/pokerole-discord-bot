use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::characters::reset_character_stats::reset_character_stats;
use crate::commands::characters::{
    log_action, reset_character_stats, update_character_post, ActionType,
};
use crate::commands::{
    find_character, pokemon_from_autocomplete_string, send_ephemeral_reply, send_error, Context,
    Error,
};
use crate::game_data::PokemonApiId;

/// Update character data.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn edit_character(
    ctx: Context<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    character: String,
    #[description = "Change their name?"] name: Option<String>,
    #[description = "Change their species?"]
    #[autocomplete = "autocomplete_pokemon"]
    species: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;

    let record = sqlx::query!(
        "SELECT name, species_api_id FROM character WHERE id = ?",
        character.id
    )
    .fetch_one(&ctx.data().database)
    .await?;

    let mut action_log = Vec::new();

    let mut should_stats_be_reset = false;
    let species = if let Some(species) = species {
        let species = pokemon_from_autocomplete_string(&ctx, &species)?;
        if species.poke_api_id.0 as i64 != record.species_api_id {
            action_log.push(format!("species to {}", species.name));
            should_stats_be_reset = true;
        }
        species
    } else {
        ctx.data()
            .game
            .pokemon_by_api_id
            .get(&PokemonApiId(record.species_api_id as u16))
            .expect("Species IDs in database should always be valid!")
    };

    let name = if let Some(name) = name {
        action_log.push(format!("name to {}", name));
        name
    } else {
        record.name
    };

    if action_log.is_empty() {
        send_error(&ctx, "No changes requested, aborting.").await?;
        return Ok(());
    }

    let _ = sqlx::query!(
        "UPDATE character SET name = ?, species_api_id = ? WHERE id = ?",
        name,
        species.poke_api_id.0,
        character.id
    )
    .execute(&ctx.data().database)
    .await;

    if should_stats_be_reset {
        let _ = reset_character_stats::reset_db_stats(&ctx, &character).await;
        action_log.push("and reset their stats".to_string());
    }

    update_character_post(&ctx, character.id).await;

    let action_log = action_log.join(", ");
    let _ = log_action(
        &ActionType::CharacterEdit,
        &ctx,
        &format!("Set {}'s {}.", character.name, action_log),
    )
    .await;
    let _ = send_ephemeral_reply(
        &ctx,
        &format!("Updated {}'s {}.", character.name, action_log),
    )
    .await;
    Ok(())
}
