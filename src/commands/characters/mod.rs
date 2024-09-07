use core::fmt;
use std::fmt::Formatter;
use std::sync::Arc;

use poise::Command;
use serenity::all::{
    ButtonStyle, CreateActionRow, CreateAllowedMentions, CreateButton, CreateMessage, GetMessages,
};
use serenity::model::id::ChannelId;
use sqlx::{Pool, Sqlite};

use crate::cache::CharacterCacheItem;
use crate::character_stats::GenericCharacterStats;
use crate::commands::{
    parse_character_names, send_error, update_character_post, BuildUpdatedStatMessageStringResult,
    Context,
};
use crate::data::Data;
use crate::enums::{Gender, MysteryDungeonRank, PokemonTypeWithoutShadow};
use crate::game_data::{GameData, PokemonApiId};
use crate::{emoji, helpers, Error};

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

const DEFAULT_BACKPACK_SLOTS: i64 = 6;

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

pub async fn send_stale_data_error<'a>(ctx: &Context<'a>) -> Result<(), Error> {
    send_error(ctx, "Something went wrong!
You hit an absolute edge case where the value has been updated by someone else while this command has been running.
If this seriously ever happens and/or turns into a problem, let me know. For now... try again? :'D
You can copy the command string either by just pressing the up key inside the text field on pc.",
    ).await
}

async fn count_completed_quests<'a>(database: &Pool<Sqlite>, character_id: i64) -> i64 {
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM quest_completion WHERE character_id = ?",
        character_id
    )
    .fetch_optional(database)
    .await;

    if let Ok(Some(record)) = result {
        record.count
    } else {
        0
    }
}

pub fn append_tera_charges(
    string: &mut String,
    pokemon_type: PokemonTypeWithoutShadow,
    unlocked: i64,
    used: i64,
) {
    if unlocked > 0 {
        string.push_str(&format!(
            "- `{}/{}` {}\n",
            unlocked - used,
            unlocked,
            pokemon_type
        ));
    }
}

pub async fn build_character_string(
    database: &Pool<Sqlite>,
    game_data: &Arc<GameData>,
    character_id: i64,
) -> Option<BuildUpdatedStatMessageStringResult> {
    let entry = sqlx::query!(
        "SELECT * FROM character WHERE id = ? \
                ORDER BY rowid \
                LIMIT 1",
        character_id,
    )
    .fetch_one(database)
    .await;

    let completed_quest_count = count_completed_quests(database, character_id).await;
    match entry {
        Ok(record) => {
            let level = helpers::calculate_level_from_experience(record.experience);
            let experience = helpers::calculate_current_experience(record.experience);
            let rank = MysteryDungeonRank::from_level(level as u8);
            let pokemon = game_data
                .pokemon_by_api_id
                .get(&PokemonApiId(
                    record
                        .species_api_id
                        .try_into()
                        .expect("Should always be in valid range."),
                ))
                .expect("All mons inside the Database should have a valid API ID assigned.");
            let gender = Gender::from_phenotype(record.phenotype);
            let emoji = emoji::get_pokemon_emoji(
                database,
                record.guild_id,
                pokemon,
                &gender,
                record.is_shiny,
            )
            .await
            .unwrap_or(format!("[{}]", pokemon.name));
            let species_override_for_stats =
                if let Some(species_override_for_stats) = record.species_override_for_stats {
                    let species_override_for_stats = game_data
                        .pokemon_by_api_id
                        .get(&PokemonApiId(species_override_for_stats as u16))?;

                    format!(
                        " | [Override: Using base stats for {}]",
                        species_override_for_stats.name
                    )
                } else {
                    String::new()
                };

            let type_emojis = if let Some(type2) = pokemon.type2 {
                format!(
                    "{}/{}",
                    emoji::type_to_emoji(&pokemon.type1),
                    emoji::type_to_emoji(&type2)
                )
            } else {
                emoji::type_to_emoji(&pokemon.type1).to_string()
            };

            let pokemon_evolution_form_for_stats = helpers::get_usual_evolution_stage_for_level(
                level,
                pokemon,
                game_data,
                record.species_override_for_stats,
            );
            let combat_stats = GenericCharacterStats::from_combat(
                pokemon_evolution_form_for_stats,
                record.stat_strength,
                record.stat_dexterity,
                record.stat_vitality,
                record.stat_special,
                record.stat_insight,
            );

            let social_stats = GenericCharacterStats::from_social(
                record.stat_tough,
                record.stat_cool,
                record.stat_beauty,
                record.stat_cute,
                record.stat_clever,
            );

            let ability_list =
                pokemon.build_simple_ability_list(record.is_hidden_ability_unlocked, false);

            let retired_or_not = if record.is_retired { "[RETIRED]" } else { "" };

            let battle_point = if record.battle_points > 0 {
                format!("\n{} {}", record.battle_points, emoji::BATTLE_POINT)
            } else {
                String::new()
            };

            let mut tera_charges = String::new();
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Bug, record.tera_unlocked_bug, record.tera_used_bug);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Dark, record.tera_unlocked_dark, record.tera_used_dark);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Dragon, record.tera_unlocked_dragon, record.tera_used_dragon);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Electric, record.tera_unlocked_electric, record.tera_used_electric);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Fairy, record.tera_unlocked_fairy, record.tera_used_fairy);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Fire, record.tera_unlocked_fire, record.tera_used_fire);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Fighting, record.tera_unlocked_fighting, record.tera_used_fighting);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Flying, record.tera_unlocked_flying, record.tera_used_flying);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Ghost, record.tera_unlocked_ghost, record.tera_used_ghost);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Grass, record.tera_unlocked_grass, record.tera_used_grass);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Ground, record.tera_unlocked_ground, record.tera_used_ground);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Ice, record.tera_unlocked_ice, record.tera_used_ice);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Normal, record.tera_unlocked_normal, record.tera_used_normal);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Poison, record.tera_unlocked_poison, record.tera_used_poison);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Psychic, record.tera_unlocked_psychic, record.tera_used_psychic);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Rock, record.tera_unlocked_rock, record.tera_used_rock);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Steel, record.tera_unlocked_steel, record.tera_used_steel);
            #[rustfmt::skip] append_tera_charges(&mut tera_charges, PokemonTypeWithoutShadow::Water, record.tera_unlocked_water, record.tera_used_water);

            if !tera_charges.is_empty() {
                tera_charges.insert_str(0, "### Terastallization Charges\n");
            }

            let mut message = format!(
                "\
## {} {} {} {}
**Level {}** `({} / 100)`
{} {} {}
### Stats {}{}
```
{}
{}
```
### Abilities 
{}{}### Statistics
{} Backpack Slots: {}\n\n",
                rank.emoji_string(),
                record.name,
                emoji,
                retired_or_not,
                level,
                experience,
                record.money,
                emoji::POKE_COIN,
                battle_point,
                type_emojis,
                species_override_for_stats,
                combat_stats.build_string(),
                social_stats.build_string(),
                ability_list,
                tera_charges,
                emoji::BACKPACK,
                record.backpack_upgrade_count + DEFAULT_BACKPACK_SLOTS,
            );

            if completed_quest_count > 0 {
                message.push_str(&format!(
                    "{} Completed Quests: {}\n",
                    emoji::TROPHY,
                    completed_quest_count
                ));
            }

            if record.total_spar_count > 0 {
                message.push_str(&format!(
                    "{} Total Sparring Sessions: {}\n",
                    emoji::FENCING,
                    record.total_spar_count
                ));
            }

            if record.total_new_player_tour_count > 0 {
                message.push_str(&format!(
                    "{} Given tours: {}\n",
                    emoji::TICKET,
                    record.total_new_player_tour_count
                ));
            }

            if record.total_new_player_combat_tutorial_count > 0 {
                message.push_str(&format!(
                    "{} Given combat tutorials: {}\n",
                    emoji::CROSSED_SWORDS,
                    record.total_new_player_combat_tutorial_count
                ));
            }

            let remaining_combat_points = helpers::calculate_available_combat_points(level)
                - combat_stats.calculate_invested_stat_points();
            let remaining_social_points = helpers::calculate_available_social_points(&rank) as i64
                - social_stats.calculate_invested_stat_points();

            let mut components = Vec::new();
            if remaining_combat_points + remaining_social_points > 0 {
                let mut action_row = Vec::new();
                if remaining_combat_points > 0 {
                    action_row.push(
                        CreateButton::new(format!("ce_initialize_combat_{}", character_id))
                            .label(format!("{} Remaining Stat Points", remaining_combat_points))
                            .style(ButtonStyle::Primary),
                    );
                }
                if remaining_social_points > 0 {
                    action_row.push(
                        CreateButton::new(format!("ce_initialize_social_{}", character_id))
                            .label(format!(
                                "{} Remaining Social Points",
                                remaining_social_points
                            ))
                            .style(ButtonStyle::Primary),
                    )
                }

                components.push(CreateActionRow::Buttons(action_row));
            }
            if completed_quest_count > 0 {
                let mut quest_history_button_row = Vec::new();
                quest_history_button_row.push(
                    CreateButton::new(format!("quest-history_{}", character_id))
                        .label("Show Quest History")
                        .style(ButtonStyle::Secondary),
                );
                components.push(CreateActionRow::Buttons(quest_history_button_row));
            }

            Some(BuildUpdatedStatMessageStringResult {
                message,
                components,
                name: record.name,
                stat_channel_id: record.stat_channel_id,
                stat_message_id: record.stat_message_id,
            })
        }
        Err(_) => None,
    }
}

#[derive(PartialEq)]
pub enum ActionType {
    Initialization,
    Reward,
    Payment,
    BackpackUpgrade,
    HiddenAbilityUnlock,
    TradeOutgoing,
    TradeIncoming,
    WalletChange,
    WalletPayment,
    WalletWithdrawal,
    Undo,
    Spar,
    NewPlayerCombatTutorial,
    NewPlayerTour,
    WalletEdit,
    CharacterEdit,
    CharacterStatReset,
    CharacterRetirement,
    CharacterUnRetirement,
    TerastallizationUnlock,
    StoreGMExperience,
    UseGMExperience,
    RuleUpdate,
    RuleDelete,
    RuleClone,
    DoNotLog,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ActionType::Initialization => "🌟 [Init]",
            ActionType::Reward => "✨ [Reward]",
            ActionType::BackpackUpgrade => "🎒 [Upgrade]",
            ActionType::HiddenAbilityUnlock => "💊 [HA Unlock]",
            ActionType::Payment => "💰 [Payment]",
            ActionType::TradeOutgoing => "➡️ [Trade]",
            ActionType::TradeIncoming => "⬅️ [Trade]",
            ActionType::WalletChange => "👛 [Wallet]",
            ActionType::WalletPayment => "👛⬅️ [Payment]",
            ActionType::WalletWithdrawal => "👛➡️ [Withdrawal]",
            ActionType::Undo => "↩️ [Undo]",
            ActionType::Spar => "🤺 [Spar]",
            ActionType::NewPlayerCombatTutorial => "⚔️ [Combat Tutorial]",
            ActionType::NewPlayerTour => "🎫 [Tour]",
            ActionType::WalletEdit => "📝 [Edit]",
            ActionType::CharacterEdit => "📝 [Edit]",
            ActionType::CharacterStatReset => "📝 [Edit]",
            ActionType::CharacterRetirement => "💤 [Retirement]",
            ActionType::CharacterUnRetirement => "⏰ [UnRetirement]",
            ActionType::TerastallizationUnlock => "💎 [Terastallization Unlock]",
            ActionType::StoreGMExperience => "🏦⬅️ [GM Experience]",
            ActionType::UseGMExperience => "🏦➡️ [GM Experience]",
            ActionType::RuleUpdate => "⚖️🌟 [Rule Update]",
            ActionType::RuleDelete => "⚖️❌ [Rule Deletion]",
            ActionType::RuleClone => "⚖️⚖️⚖️ [Rule Clone]",
            ActionType::DoNotLog => "",
        })
    }
}

pub async fn log_action<'a>(
    action_type: &ActionType,
    ctx: &Context<'a>,
    message: impl Into<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id();
    if guild_id.is_none() {
        return Ok(());
    }

    let guild_id = guild_id.expect("should only be called in guild_only").get() as i64;
    let record = sqlx::query!(
        "SELECT action_log_channel_id FROM guild WHERE id = ?",
        guild_id
    )
    .fetch_one(&ctx.data().database)
    .await;

    let origin = match ctx
        .channel_id()
        .messages(ctx, GetMessages::new().limit(1))
        .await
    {
        Ok(messages) => match messages.first() {
            None => String::new(),
            Some(m) => format!(" in {}", m.link_ensured(ctx).await),
        },
        Err(_) => String::new(),
    };

    if let Ok(record) = record {
        if let Some(action_log_channel_id) = record.action_log_channel_id {
            let channel_id = ChannelId::from(action_log_channel_id as u64);
            channel_id
                .send_message(
                    ctx,
                    CreateMessage::new()
                        .content(std::format!(
                            "{} {} (triggered by {}{})",
                            action_type,
                            message.into(),
                            ctx.author(),
                            origin
                        ))
                        .allowed_mentions(CreateAllowedMentions::new().empty_users()),
                )
                .await?;
        }
    }

    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct EntityWithNameAndNumericValue {
    pub id: i64,
    pub name: String,
    pub value: i64,
}

pub async fn change_character_stat<'a>(
    ctx: &Context<'a>,
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
    ctx: &Context<'a>,
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

            update_character_post(ctx, record.id).await;
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
                    let old_level = helpers::calculate_level_from_experience(record.value);
                    let new_level = helpers::calculate_level_from_experience(record.value + amount);
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
                log_action(action_type, ctx, format!("{} {} {} {} {}", added_or_removed, amount.abs(), action, to_or_from, record.name).as_str()).await
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
    helpers::validate_user_input(text, 30)
}

pub fn build_character_list(characters: &[CharacterCacheItem]) -> String {
    characters
        .iter()
        .map(|x| x.name.as_str())
        .collect::<Vec<&str>>()
        .join(", ")
}
