use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::{Error, find_character};
use crate::shared::PoiseContext;
use crate::shared::action_log::{ActionType, LogActionArguments, log_action};
use crate::shared::cache::CharacterCacheItem;
use crate::shared::character::update_character_post_with_poise_context;
use crate::shared::game_data::PokemonApiId;
use crate::shared::utility::level_calculations;

/// Resets a characters stats to its default values.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn reset_character_stats(
    ctx: PoiseContext<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    character: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;

    reset_db_stats(&ctx, &character).await?;
    update_character_post_with_poise_context(&ctx, character.id).await;

    let _ = ctx
        .reply(&format!("{}'s stats have been reset.", character.name))
        .await;
    let _ = log_action(
        &ActionType::CharacterStatReset,
        LogActionArguments::triggered_by_user(&ctx),
        &format!("Reset {}'s stats", character.name),
    )
    .await;
    Ok(())
}

pub async fn reset_db_stats(
    ctx: &PoiseContext<'_>,
    character: &CharacterCacheItem,
) -> Result<(), Error> {
    let record = sqlx::query!(
        "SELECT name, species_api_id, experience, species_override_for_stats FROM character WHERE id = ?",
        character.id
    )
        .fetch_one(&ctx.data().database)
        .await?;

    let species_id = PokemonApiId(record.species_api_id as u16);
    let used_poke_species = ctx
        .data()
        .game
        .base_data
        .pokemon_by_api_id
        .get(&species_id)
        .expect("DB IDs should always be mappable.");

    let level = level_calculations::calculate_level_from_experience(record.experience);

    let pokemon_evolution_form_for_stats = level_calculations::get_usual_evolution_stage_for_level(
        level,
        used_poke_species,
        &ctx.data().game.base_data,
        record.species_override_for_stats,
    );

    let _ = sqlx::query!(
        "UPDATE character SET 
stat_strength = ?,
stat_dexterity = ?,
stat_vitality = ?,
stat_special = ?,
stat_insight = ?,
stat_edit_strength = ?,
stat_edit_dexterity = ?,
stat_edit_vitality = ?,
stat_edit_special = ?,
stat_edit_insight = ?,
stat_tough = 1,
stat_cool = 1,
stat_beauty = 1,
stat_cute = 1,
stat_clever = 1,
stat_edit_tough = 1,
stat_edit_cool = 1,
stat_edit_beauty = 1,
stat_edit_cute = 1,
stat_edit_clever = 1
    WHERE id = ?",
        pokemon_evolution_form_for_stats.strength.min,
        pokemon_evolution_form_for_stats.dexterity.min,
        pokemon_evolution_form_for_stats.vitality.min,
        pokemon_evolution_form_for_stats.special.min,
        pokemon_evolution_form_for_stats.insight.min,
        pokemon_evolution_form_for_stats.strength.min,
        pokemon_evolution_form_for_stats.dexterity.min,
        pokemon_evolution_form_for_stats.vitality.min,
        pokemon_evolution_form_for_stats.special.min,
        pokemon_evolution_form_for_stats.insight.min,
        character.id
    )
    .execute(&ctx.data().database)
    .await;
    Ok(())
}
