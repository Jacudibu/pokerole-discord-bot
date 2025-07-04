use crate::Error;
use crate::commands::{parse_character_names, send_error};
use crate::shared::action_log::{ActionType, LogActionArguments};
use crate::shared::cache::CharacterCacheItem;
use crate::shared::character::update_character_post_with_poise_context;
use crate::shared::data::Data;
use crate::shared::enums::{MysteryDungeonRank, PokemonTypeWithoutShadow};
use crate::shared::utility::{input_validation, level_calculations};
use crate::shared::{PoiseContext, action_log, emoji};
use poise::Command;

mod character_sheet;
mod create_character;
mod create_character_post;
mod edit_character;
mod give_money;
mod reset_character_stats;
mod retire_character;
mod reward_battle_points;
mod reward_experience;
mod reward_giving_combat_tutorial;
mod reward_giving_tour;
mod reward_money;
mod reward_spar;
mod reward_terastallization;
mod unlock_hidden_ability;
mod unretire_character;
mod upgrade_backpack;
mod use_terastallization;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![
        character_sheet::character_sheet(),
        edit_character::edit_character(),
        give_money::give_money(),
        create_character::create_character(),
        create_character::initialize_character(),
        create_character_post::create_character_post(),
        reward_experience::reward_experience(),
        reward_money::reward_money(),
        upgrade_backpack::upgrade_backpack(),
        unlock_hidden_ability::unlock_hidden_ability(),
        reward_battle_points::reward_battle_points(),
        reward_spar::reward_spar(),
        reward_giving_combat_tutorial::reward_giving_combat_tutorial(),
        reward_giving_tour::reward_giving_tour(),
        reset_character_stats::reset_character_stats(),
        retire_character::retire_character(),
        unretire_character::unretire_character(),
        use_terastallization::use_terastallization(),
        reward_terastallization::reward_terastallization(),
    ]
}

// /// Trigger an update for all character sheets.
// #[poise::command(
//     slash_command,
//     guild_only,
//     default_member_permissions = "ADMINISTRATOR"
// )]
// async fn update_all_character_posts(ctx: Context<'_>) -> Result<(), Error> {
//     if ctx.author().id.get() != ADMIN_ID {
//         return send_error(
//             &ctx,
//             &format!(
//                 "Sorry, but this command is so unbelievably spam-inducing that it's only available for {}.",
//                 ADMIN_PING_STRING
//             ),
//         )
//         .await;
//     }
//
//     let _ = ctx.defer_ephemeral().await;
//     for record in sqlx::query!("SELECT id from character")
//         .fetch_all(&ctx.data().database)
//         .await
//         .unwrap()
//     {
//         update_character_post(&ctx, record.id).await;
//     }
//
//     let _ = send_ephemeral_reply(&ctx, "Done!").await;
//     Ok(())
// }

pub async fn send_stale_data_error<'a>(ctx: &PoiseContext<'a>) -> Result<(), Error> {
    send_error(ctx, "Something went wrong!
You hit an absolute edge case where the value has been updated by someone else while this command has been running.
If this seriously ever happens and/or turns into a problem, let me know. For now... try again? :'D
You can copy the command string either by just pressing the up key inside the text field on pc.",
    ).await
}

#[derive(sqlx::FromRow)]
pub struct EntityWithNameAndNumericValue {
    pub id: i64,
    pub name: String,
    pub value: i64,
}

pub async fn change_character_stat<'a>(
    ctx: &PoiseContext<'a>,
    database_column: &str,
    names: &Vec<String>,
    amount: i64,
    action_type: ActionType,
) -> Result<Vec<CharacterCacheItem>, String> {
    let guild_id = ctx
        .guild_id()
        .expect("Commands using this function are marked as guild_only")
        .get();

    let characters = parse_character_names(ctx, guild_id, names).await?;
    for x in &characters {
        let _ =
            change_character_stat_after_validation(ctx, database_column, x, amount, &action_type)
                .await;
    }

    Ok(characters)
}

pub async fn change_character_stat_after_validation<'a>(
    ctx: &PoiseContext<'a>,
    database_column: &str,
    character: &CharacterCacheItem,
    amount: i64,
    action_type: &ActionType,
) -> Result<(), Error> {
    if action_type != &ActionType::DoNotLog {
        // Replying should be handled before calling this in DoNotLog scenarios.
        ctx.defer().await?;
    }
    let record = sqlx::query_as::<_, EntityWithNameAndNumericValue>(
        format!(
            "SELECT id, name, {} as value FROM character WHERE id = ?",
            database_column
        )
        .as_str(),
    )
    .bind(character.id)
    .fetch_one(&ctx.data().database)
    .await;

    match record {
        Ok(record) => {
            let new_value = record.value + amount;
            let result = sqlx::query(
                format!("UPDATE character SET {} = ? WHERE id = ? AND {} = ?", database_column, database_column).as_str())
                .bind(new_value)
                .bind(record.id)
                .bind(record.value)
                .execute(&ctx.data().database).await;

            if result.is_err() || result.unwrap().rows_affected() != 1 {
                return send_stale_data_error(ctx).await;
            }

            update_character_post_with_poise_context(ctx, record.id).await;
            let action = match database_column {
                "money" => String::from(emoji::POKE_COIN),
                "battle_points" => String::from(emoji::BATTLE_POINT),
                "tera_unlocked_normal" => PokemonTypeWithoutShadow::Normal.to_string(),
                "tera_unlocked_fighting" => PokemonTypeWithoutShadow::Fighting.to_string(),
                "tera_unlocked_flying" => PokemonTypeWithoutShadow::Flying.to_string(),
                "tera_unlocked_poison" => PokemonTypeWithoutShadow::Poison.to_string(),
                "tera_unlocked_ground" => PokemonTypeWithoutShadow::Ground.to_string(),
                "tera_unlocked_rock" => PokemonTypeWithoutShadow::Rock.to_string(),
                "tera_unlocked_bug" => PokemonTypeWithoutShadow::Bug.to_string(),
                "tera_unlocked_ghost" => PokemonTypeWithoutShadow::Ghost.to_string(),
                "tera_unlocked_steel" => PokemonTypeWithoutShadow::Steel.to_string(),
                "tera_unlocked_fire" => PokemonTypeWithoutShadow::Fire.to_string(),
                "tera_unlocked_water" => PokemonTypeWithoutShadow::Water.to_string(),
                "tera_unlocked_grass" => PokemonTypeWithoutShadow::Grass.to_string(),
                "tera_unlocked_electric" => PokemonTypeWithoutShadow::Electric.to_string(),
                "tera_unlocked_psychic" => PokemonTypeWithoutShadow::Psychic.to_string(),
                "tera_unlocked_ice" => PokemonTypeWithoutShadow::Ice.to_string(),
                "tera_unlocked_dragon" => PokemonTypeWithoutShadow::Dragon.to_string(),
                "tera_unlocked_dark" => PokemonTypeWithoutShadow::Dark.to_string(),
                "tera_unlocked_fairy" => PokemonTypeWithoutShadow::Fairy.to_string(),
                _ => String::from(database_column)
            };
            let added_or_removed: &str;
            let to_or_from: &str;
            if amount > 0 {
                added_or_removed = "Added";
                to_or_from = "to";

                if database_column == "experience" {
                    let old_level = level_calculations::calculate_level_from_experience(record.value);
                    let new_level = level_calculations::calculate_level_from_experience(record.value + amount);
                    if new_level > old_level {
                        let old_rank = MysteryDungeonRank::from_level(old_level as u8);
                        let new_rank = MysteryDungeonRank::from_level(new_level as u8);

                        let rank_notification = if new_rank > old_rank {
                            format!(" They are now {}!", new_rank)
                        } else {
                            String::new()
                        };

                        let _ = ctx.say(format!("### {} Level Up! {}\n**{}** just reached level {}!{}", emoji::PARTY_POPPER, emoji::PARTYING_FACE, record.name, new_level, rank_notification)).await;
                    }
                }
            } else {
                added_or_removed = "Removed";
                to_or_from = "from";
            }

            if action_type != &ActionType::DoNotLog {
                action_log::log_action(action_type,
                                       LogActionArguments::triggered_by_user(&ctx),
                                       format!("{} {} {} {} {}", added_or_removed, amount.abs(), action, to_or_from, record.name).as_str()).await
            } else {
                Ok(())
            }
        }
        Err(_) => {
            send_error(ctx, format!("Unable to find a character named {}.\n**Internal cache must be out of date. Please let me know if this ever happens.**", character.name).as_str()).await
        }
    }
}

pub fn validate_user_input<'a>(text: &str) -> Result<(), &'a str> {
    input_validation::validate_user_input(text, 30)
}

pub fn build_character_list(characters: &[CharacterCacheItem]) -> String {
    characters
        .iter()
        .map(|x| x.name.as_str())
        .collect::<Vec<&str>>()
        .join(", ")
}
