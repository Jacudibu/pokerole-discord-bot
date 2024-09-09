use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::autocompletion::autocomplete_pokemon_type;
use crate::commands::characters::{log_action, reset_character_stats, ActionType};
use crate::commands::create_emojis::create_emojis_for_pokemon;
use crate::commands::{
    ensure_user_exists, find_character, pokemon_from_autocomplete_string, send_ephemeral_reply,
    update_character_post, Context, Error,
};
use crate::enums::{Gender, PokemonTypeWithoutShadow};
use crate::errors::ValidationError;
use crate::game_data::PokemonApiId;
use serenity::all::User;
use serenity::prelude::Mentionable;

/// Update character data. All arguments are optional.
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
    #[description = "Change from which pokemon their stats should be taken? Useful for unevolved mons at high levels."]
    #[autocomplete = "autocomplete_pokemon"]
    species_override_for_stats: Option<String>,
    #[description = "Change the characters' shiny state."] is_shiny: Option<bool>,
    #[description = "Change Tera Charges for specific Type. Also set tera_count."]
    #[autocomplete = "autocomplete_pokemon_type"]
    tera_type: Option<PokemonTypeWithoutShadow>,
    #[description = "Change Tera Charges for specific Type. Also set tera_type."]
    #[min = 0_i64]
    tera_count: Option<i64>,
    #[description = "Transfer ownership to another player."] new_owner: Option<User>,
) -> Result<(), Error> {
    if species.is_some() && species_override_for_stats.is_some() {
        return Err(Box::new(ValidationError::new("\
You can't (and probably also don't really want to) edit a character's species and its override at the same time:
- Changing the species will remove any existing overrides.
- Overrides are meant for situations where you aren't able to change the species due to to other external constraints."
        )));
    }

    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;

    let record = sqlx::query!(
        "SELECT user_id, name, species_api_id, phenotype, is_shiny, species_override_for_stats FROM character WHERE id = ?",
        character.id
    )
        .fetch_one(&ctx.data().database)
        .await?;

    let mut action_log = Vec::new();
    let mut create_emojis = false;

    let gender = Gender::from_phenotype(record.phenotype);

    let mut invalidate_cache = false;
    let mut should_stats_be_reset = false;
    let mut reset_species_override = false;
    let species = if let Some(species) = species {
        let species = pokemon_from_autocomplete_string(&ctx, &species).await?;
        if species.poke_api_id.0 as i64 != record.species_api_id {
            action_log.push(format!("species to {}", species.name));
            should_stats_be_reset = record.species_override_for_stats.is_none();
        }

        create_emojis = true;
        if let Some(existing_species_override) = record.species_override_for_stats {
            if existing_species_override == species.poke_api_id.0 as i64 {
                reset_species_override = true;
                should_stats_be_reset = false;
            }
        }

        species
    } else {
        ctx.data()
            .game
            .base_data
            .pokemon_by_api_id
            .get(&PokemonApiId(record.species_api_id as u16))
            .expect("Species IDs in database should always be valid!")
    };

    let species_override_for_stats = if let Some(species_override_for_stats) =
        species_override_for_stats
    {
        let species = pokemon_from_autocomplete_string(&ctx, &species_override_for_stats).await?;
        if species.poke_api_id.0 as i64 != record.species_api_id {
            action_log.push(format!("species stat override to {}", species.name));
            should_stats_be_reset = true;
            Some(species.poke_api_id.0 as i64)
        } else {
            action_log.push(String::from("removed species stat override"));
            None
        }
    } else if reset_species_override {
        action_log.push(String::from("removed species stat override"));
        None
    } else {
        record.species_override_for_stats
    };

    let name = if let Some(name) = name {
        action_log.push(format!("name to {}", name));
        name
    } else {
        record.name
    };

    let is_shiny = if let Some(is_shiny) = is_shiny {
        action_log.push(format!("is_shiny to {}", is_shiny));
        create_emojis = true;
        is_shiny
    } else {
        record.is_shiny
    };

    if let Some(tera_type) = tera_type {
        if let Some(tera_count) = tera_count {
            action_log.push(format!(
                "{} Terastallization count to {}",
                tera_type, tera_count
            ));
        } else {
            return Err(Box::new(ValidationError::new("To set tera_type, also set tera_count to the amount of tera charges that type should have.")));
        }
    } else if tera_count.is_some() {
        return Err(Box::new(ValidationError::new(
            "To set tera_count, also set tera_type so I know which type you want to change.",
        )));
    }

    let user_id = if let Some(new_owner) = new_owner {
        action_log.push(format!("owner to {}", new_owner.mention()));
        invalidate_cache = true;
        let user_id = new_owner.id.get() as i64;
        ensure_user_exists(&ctx, user_id, guild_id as i64).await;
        user_id
    } else {
        record.user_id
    };

    if action_log.is_empty() {
        return Err(Box::new(ValidationError::new(
            "No changes requested, aborting.",
        )));
    }

    if create_emojis {
        create_emojis_for_pokemon(&ctx, species, &gender, is_shiny).await;
    }

    sqlx::query!(
        "UPDATE character SET name = ?, species_api_id = ?, species_override_for_stats = ?, is_shiny = ?, user_id = ? WHERE id = ?",
        name,
        species.poke_api_id.0,
        species_override_for_stats,
        is_shiny,
        user_id,
        character.id,
    )
        .execute(&ctx.data().database)
        .await?;

    if let Some(tera_type) = tera_type {
        if let Some(tera_count) = tera_count {
            let column = tera_type.get_tera_unlocked_column();
            sqlx::query(&format!("UPDATE character SET {} = ? WHERE id = ?", column))
                .bind(tera_count)
                .bind(character.id)
                .execute(&ctx.data().database)
                .await?;
        }
    }

    if should_stats_be_reset {
        let _ = reset_character_stats::reset_db_stats(&ctx, &character).await;
        action_log.push("and reset their stats".to_string());
    }

    if invalidate_cache {
        ctx.data()
            .cache
            .update_character_names(&ctx.data().database)
            .await;
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
