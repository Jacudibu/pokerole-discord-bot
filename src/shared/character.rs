use crate::commands::BuildUpdatedStatMessageStringResult;
use crate::shared::character_stats::GenericCharacterStats;
use crate::shared::data::Data;
use crate::shared::enums::{Gender, MysteryDungeonRank, PokemonTypeWithoutShadow};
use crate::shared::game_data::{GameData, PokemonApiId};
use crate::shared::utility::{error_handling, level_calculations};
use crate::shared::{PoiseContext, SerenityContext, constants, emoji};
use serenity::all::{
    ButtonStyle, ChannelId, CreateActionRow, CreateButton, EditMessage, GuildId, MessageId, UserId,
};
use sqlx::{Pool, Sqlite};

pub async fn build_character_string(
    context: &serenity::all::Context,
    database: &Pool<Sqlite>,
    game_data: &GameData,
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
            let level = level_calculations::calculate_level_from_experience(record.experience);
            let experience = level_calculations::calculate_current_experience(record.experience);
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
                context,
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

            let type_emojis = if let Some(type2) = pokemon.types.type2 {
                format!(
                    "{}/{}",
                    emoji::type_to_emoji(&pokemon.types.type1),
                    emoji::type_to_emoji(&type2)
                )
            } else {
                emoji::type_to_emoji(&pokemon.types.type1).to_string()
            };

            let pokemon_evolution_form_for_stats =
                level_calculations::get_usual_evolution_stage_for_level(
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

            let ability_list = pokemon
                .abilities
                .build_simple_ability_list(record.is_hidden_ability_unlocked, false);

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
                record.backpack_upgrade_count + constants::DEFAULT_BACKPACK_SLOTS,
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

            let remaining_combat_points =
                level_calculations::calculate_available_combat_points(level)
                    - combat_stats.calculate_invested_stat_points();
            let remaining_social_points =
                rank.social_stat_points() as i64 - social_stats.calculate_invested_stat_points();

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

fn append_tera_charges(
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

pub async fn update_character_post_with_serenity_context(
    context: &SerenityContext,
    guild_id: Option<GuildId>,
    channel_id: Option<ChannelId>,
    owner_id: UserId,
    data: &Data,
    character_id: i64,
) {
    if let Some(result) = build_character_string(
        context,
        &data.database,
        data.game.get(guild_id, owner_id, &data.database).await,
        character_id,
    )
    .await
    {
        let message = context
            .http
            .get_message(
                ChannelId::from(result.stat_channel_id as u64),
                MessageId::from(result.stat_message_id as u64),
            )
            .await;
        if let Ok(mut message) = message {
            if let Err(e) = message
                .edit(
                    context,
                    EditMessage::new()
                        .content(&result.message)
                        .components(result.components.clone()),
                )
                .await
            {
                error_handling::handle_error_during_message_edit(
                    context,
                    e,
                    message,
                    result.message,
                    Some(result.components),
                    result.name,
                    channel_id,
                )
                .await;
            }
        }
    }
}

pub async fn update_character_post_with_poise_context<'a>(ctx: &PoiseContext<'a>, id: i64) {
    update_character_post_with_serenity_context(
        ctx.serenity_context(),
        ctx.guild_id(),
        Some(ctx.channel_id()),
        ctx.author().id,
        ctx.data(),
        id,
    )
    .await;
}
