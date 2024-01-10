use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::characters::{log_action, update_character_post, ActionType};
use crate::commands::{find_character, Context, Error};
use crate::game_data::PokemonApiId;

/// Resets a characters stats to its default values.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn reset_character_stats(
    ctx: Context<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_character_name"]
    character: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;

    let record = sqlx::query!(
        "SELECT name, species_api_id FROM character WHERE id = ?",
        character.id
    )
    .fetch_one(&ctx.data().database)
    .await?;

    let species_id = PokemonApiId(record.species_api_id as u16);
    let species = ctx
        .data()
        .game
        .pokemon_by_api_id
        .get(&species_id)
        .expect("DB IDs should always be mappable.");

    // TODO: Account for pre-evolution stats if mons evolve earlier.

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
        species.strength.min,
        species.dexterity.min,
        species.vitality.min,
        species.special.min,
        species.insight.min,
        species.strength.min,
        species.dexterity.min,
        species.vitality.min,
        species.special.min,
        species.insight.min,
        character.id
    )
    .execute(&ctx.data().database)
    .await;

    update_character_post(&ctx, character.id).await;

    let _ = ctx
        .reply(&format!("{}'s stats have been reset.", character.name))
        .await;
    let _ = log_action(
        &ActionType::CharacterStatReset,
        &ctx,
        &format!("Reset {}'s stats", character.name),
    )
    .await;
    Ok(())
}
