use crate::commands::autocompletion::{autocomplete_character_name, autocomplete_pokemon_type};
use crate::commands::character_commands::{build_character_list, change_character_stat};
use crate::commands::{Error, parse_variadic_args};
use crate::shared::PoiseContext;
use crate::shared::action_log::ActionType;
use crate::shared::cache::CharacterCacheItem;
use crate::shared::enums::PokemonTypeWithoutShadow;
use crate::shared::errors::CommandInvocationError;

/// Reward players with a Terastallization charge.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn reward_terastallization(
    ctx: PoiseContext<'_>,
    #[autocomplete = "autocomplete_pokemon_type"]
    #[description = "Which Type?"]
    tera_type: PokemonTypeWithoutShadow,
    #[description = "Whom?"]
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

    match handle_unlock(&ctx, tera_type, &args).await {
        Ok(characters) => {
            ctx.say(format!(
                "Unlocked a {} Terastallization Charge for {}!",
                tera_type,
                build_character_list(&characters)
            ))
            .await?;
            Ok(())
        }
        Err(err) => Err(Box::new(err)),
    }
}

async fn handle_unlock<'a>(
    ctx: &PoiseContext<'a>,
    tera_type: PokemonTypeWithoutShadow,
    names: &Vec<String>,
) -> Result<Vec<CharacterCacheItem>, CommandInvocationError> {
    let tera_unlocked_column = match tera_type {
        PokemonTypeWithoutShadow::Normal => "tera_unlocked_normal",
        PokemonTypeWithoutShadow::Fighting => "tera_unlocked_fighting",
        PokemonTypeWithoutShadow::Flying => "tera_unlocked_flying",
        PokemonTypeWithoutShadow::Poison => "tera_unlocked_poison",
        PokemonTypeWithoutShadow::Ground => "tera_unlocked_ground",
        PokemonTypeWithoutShadow::Rock => "tera_unlocked_rock",
        PokemonTypeWithoutShadow::Bug => "tera_unlocked_bug",
        PokemonTypeWithoutShadow::Ghost => "tera_unlocked_ghost",
        PokemonTypeWithoutShadow::Steel => "tera_unlocked_steel",
        PokemonTypeWithoutShadow::Fire => "tera_unlocked_fire",
        PokemonTypeWithoutShadow::Water => "tera_unlocked_water",
        PokemonTypeWithoutShadow::Grass => "tera_unlocked_grass",
        PokemonTypeWithoutShadow::Electric => "tera_unlocked_electric",
        PokemonTypeWithoutShadow::Psychic => "tera_unlocked_psychic",
        PokemonTypeWithoutShadow::Ice => "tera_unlocked_ice",
        PokemonTypeWithoutShadow::Dragon => "tera_unlocked_dragon",
        PokemonTypeWithoutShadow::Dark => "tera_unlocked_dark",
        PokemonTypeWithoutShadow::Fairy => "tera_unlocked_fairy",
    };

    match change_character_stat(
        ctx,
        tera_unlocked_column,
        names,
        1,
        ActionType::TerastallizationUnlock,
    )
    .await
    {
        Ok(characters) => Ok(characters),
        Err(err) => Err(CommandInvocationError::new(&err)),
    }
}
