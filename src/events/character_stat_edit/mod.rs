mod initialize;

use crate::character_stats::GenericCharacterStats;
use crate::data::Data;
use crate::enums::{CombatOrSocialStat, Gender, MysteryDungeonRank};
use crate::events::{character_stat_edit, send_error};
use crate::game_data::PokemonApiId;
use crate::{emoji, helpers, Error};
use serenity::all::{
    ButtonStyle, ComponentInteraction, CreateActionRow, CreateInteractionResponse,
    CreateInteractionResponseMessage, ReactionType,
};
use serenity::builder::CreateButton;
use serenity::client::Context;

pub async fn handle_character_editor_command(
    context: &Context,
    interaction: &ComponentInteraction,
    data: &Data,
    mut args: Vec<&str>,
) -> Result<(), Error> {
    match args.remove(0) {
        "initialize" => initialize::initialize(context, interaction, data, args).await,
        &_ => {send_error(&interaction, context, "Seems like you are either trying to do something that's not yet implemented or that you are doing something fishy. Mhhhm~").await}
    }
}

#[derive(PartialOrd, PartialEq)]
enum StatType {
    Combat,
    Social,
}

async fn reset_stat_edit_values(data: &Data, character_id: i64) {
    let _ = sqlx::query!(
        "UPDATE character SET 
stat_edit_strength = stat_strength,
stat_edit_dexterity = stat_dexterity,
stat_edit_vitality = stat_vitality,
stat_edit_special = stat_special,
stat_edit_insight = stat_insight,
stat_edit_tough = stat_tough,
stat_edit_cool = stat_cool,
stat_edit_beauty = stat_beauty,
stat_edit_cute = stat_cute,
stat_edit_clever = stat_clever
WHERE id = ?",
        character_id
    )
    .execute(&data.database)
    .await;
}

fn create_combat_buttons(character_id: i64) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_combat-stat_add_{}_str", character_id))
                .label("+STR")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_combat-stat_add_{}_dex", character_id))
                .label("+DEX")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_combat-stat_add_{}_vit", character_id))
                .label("+VIT")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_combat-stat_add_{}_spe", character_id))
                .label("+SPE")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_combat-stat_add_{}_ins", character_id))
                .label("+INS")
                .style(ButtonStyle::Success),
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_combat-stat_subtract_{}_str", character_id))
                .label("-STR")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_combat-stat_subtract_{}_dex", character_id))
                .label("-DEX")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_combat-stat_subtract_{}_vit", character_id))
                .label("-VIT")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_combat-stat_subtract_{}_spe", character_id))
                .label("-SPE")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_combat-stat_subtract_{}_ins", character_id))
                .label("-INS")
                .style(ButtonStyle::Danger),
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_combat-stat_apply_{}", character_id))
                .label("Apply")
                .emoji(ReactionType::Unicode(emoji::UNICODE_CHECK_MARK.to_string()))
                .style(ButtonStyle::Primary),
            CreateButton::new(format!("ce_combat-stat_cancel_{}", character_id))
                .label("Cancel")
                .emoji(ReactionType::Unicode(emoji::UNICODE_CROSS_MARK.to_string()))
                .style(ButtonStyle::Secondary),
        ]),
    ]
}

fn create_social_buttons(character_id: i64) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_social-stat_add_{}_tough", character_id))
                .label("+Tough")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_social-stat_add_{}_cool", character_id))
                .label("+Cool")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_social-stat_add_{}_beauty", character_id))
                .label("+Beauty")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_social-stat_add_{}_cute", character_id))
                .label("+Cute")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_social-stat_add_{}_clever", character_id))
                .label("+Clever")
                .style(ButtonStyle::Success),
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_social-stat_subtract_{}_tough", character_id))
                .label("-Tough")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_social-stat_subtract_{}_cool", character_id))
                .label("-Cool")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_social-stat_subtract_{}_beauty", character_id))
                .label("-Beauty")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_social-stat_subtract_{}_cute", character_id))
                .label("-Cute")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_social-stat_subtract_{}_clever", character_id))
                .label("-Clever")
                .style(ButtonStyle::Danger),
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_social-stat_apply_{}", character_id))
                .label("Apply")
                .emoji(ReactionType::Unicode(emoji::UNICODE_CHECK_MARK.to_string()))
                .style(ButtonStyle::Primary),
            CreateButton::new(format!("ce_social-stat_cancel_{}", character_id))
                .label("Cancel")
                .emoji(ReactionType::Unicode(emoji::UNICODE_CROSS_MARK.to_string()))
                .style(ButtonStyle::Secondary),
        ]),
    ]
}

async fn create_stat_edit_overview_message(
    data: &Data,
    character_id: i64,
    stat_type: StatType,
) -> CreateInteractionResponseMessage {
    let record = sqlx::query!(
        "SELECT name, guild_id, experience, species_api_id, is_shiny, phenotype, \
                      stat_edit_strength, stat_edit_dexterity, stat_edit_vitality, stat_edit_special, stat_edit_insight, stat_edit_tough, stat_edit_cool, stat_edit_beauty, stat_edit_cute, stat_edit_clever
                FROM character WHERE id = ? \
                ORDER BY rowid \
                LIMIT 1",
        character_id,
    )
        .fetch_one(&data.database)
        .await
        .unwrap();

    let level = helpers::calculate_level_from_experience(record.experience);
    let experience = record.experience % 100;
    let rank = MysteryDungeonRank::from_level(level as u8);
    let pokemon = data
        .game
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
        &data.database,
        record.guild_id,
        pokemon,
        &gender,
        record.is_shiny,
    )
    .await
    .unwrap_or(format!("[{}]", pokemon.name));

    let (stats, remaining_points) = match stat_type {
        StatType::Combat => {
            let pokemon_evolution_form_for_stats =
                helpers::get_usual_evolution_stage_for_level(level, pokemon, data);
            let combat_stats = GenericCharacterStats::from_combat(
                pokemon_evolution_form_for_stats,
                record.stat_edit_strength,
                record.stat_edit_dexterity,
                record.stat_edit_vitality,
                record.stat_edit_special,
                record.stat_edit_insight,
            );

            let remaining_points = helpers::calculate_available_combat_points(level)
                - combat_stats.calculate_invested_stat_points();

            (combat_stats, remaining_points)
        }
        StatType::Social => {
            let social_stats = GenericCharacterStats::from_social(
                record.stat_edit_tough,
                record.stat_edit_cool,
                record.stat_edit_beauty,
                record.stat_edit_cute,
                record.stat_edit_clever,
            );

            let remaining_points = helpers::calculate_available_social_points(&rank) as i64
                - social_stats.calculate_invested_stat_points();

            (social_stats, remaining_points)
        }
    };

    let message = format!(
        "### {}{}\n```\n{}```\n{} Remaining Points.",
        emoji,
        record.name,
        stats.build_string(),
        remaining_points
    );

    CreateInteractionResponseMessage::new()
        .content(message)
        .ephemeral(true)
        .components(match stat_type {
            StatType::Combat => create_combat_buttons(character_id),
            StatType::Social => create_social_buttons(character_id),
        })
}
