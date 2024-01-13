mod initialize;
mod stat_edit;

use crate::character_stats::GenericCharacterStats;
use crate::data::Data;
use crate::enums::{Gender, MysteryDungeonRank};
use crate::events::send_error;
use crate::game_data::PokemonApiId;
use crate::{emoji, helpers, Error};
use serenity::all::{
    ButtonStyle, ChannelId, ComponentInteraction, CreateActionRow,
    CreateInteractionResponseMessage, EditInteractionResponse, EditMessage, MessageId,
    ReactionType,
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
        "edit-stat" => stat_edit::handle_edit_stat_request(context, interaction, data, args).await,
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
            CreateButton::new(format!("ce_edit-stat_{}_add_strength", character_id))
                .label("+STR")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_edit-stat_{}_add_dexterity", character_id))
                .label("+DEX")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_edit-stat_{}_add_vitality", character_id))
                .label("+VIT")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_edit-stat_{}_add_special", character_id))
                .label("+SPE")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_edit-stat_{}_add_insight", character_id))
                .label("+INS")
                .style(ButtonStyle::Success),
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_edit-stat_{}_subtract_strength", character_id))
                .label("-STR")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_edit-stat_{}_subtract_dexterity", character_id))
                .label("-DEX")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_edit-stat_{}_subtract_vitality", character_id))
                .label("-VIT")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_edit-stat_{}_subtract_special", character_id))
                .label("-SPE")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_edit-stat_{}_subtract_insight", character_id))
                .label("-INS")
                .style(ButtonStyle::Danger),
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_edit-stat_{}_apply-combat", character_id))
                .label("Apply")
                .emoji(ReactionType::Unicode(emoji::UNICODE_CHECK_MARK.to_string()))
                .style(ButtonStyle::Primary),
            CreateButton::new(format!("ce_edit-stat_{}_cancel", character_id))
                .label("Cancel")
                .emoji(ReactionType::Unicode(emoji::UNICODE_CROSS_MARK.to_string()))
                .style(ButtonStyle::Secondary),
        ]),
    ]
}

fn create_social_buttons(character_id: i64) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_edit-stat_{}_add_tough", character_id))
                .label("+Tough")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_edit-stat_{}_add_cool", character_id))
                .label("+Cool")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_edit-stat_{}_add_beauty", character_id))
                .label("+Beauty")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_edit-stat_{}_add_cute", character_id))
                .label("+Cute")
                .style(ButtonStyle::Success),
            CreateButton::new(format!("ce_edit-stat_{}_add_clever", character_id))
                .label("+Clever")
                .style(ButtonStyle::Success),
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_edit-stat_{}_subtract_tough", character_id))
                .label("-Tough")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_edit-stat_{}_subtract_cool", character_id))
                .label("-Cool")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_edit-stat_{}_subtract_beauty", character_id))
                .label("-Beauty")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_edit-stat_{}_subtract_cute", character_id))
                .label("-Cute")
                .style(ButtonStyle::Danger),
            CreateButton::new(format!("ce_edit-stat_{}_subtract_clever", character_id))
                .label("-Clever")
                .style(ButtonStyle::Danger),
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new(format!("ce_edit-stat_{}_apply-social", character_id))
                .label("Apply")
                .emoji(ReactionType::Unicode(emoji::UNICODE_CHECK_MARK.to_string()))
                .style(ButtonStyle::Primary),
            CreateButton::new(format!("ce_edit-stat_{}_cancel", character_id))
                .label("Cancel")
                .emoji(ReactionType::Unicode(emoji::UNICODE_CROSS_MARK.to_string()))
                .style(ButtonStyle::Secondary),
        ]),
    ]
}

struct CharacterDataForStatEditing {
    id: i64,
    name: String,
    emoji: String,
    level: i64,
    rank: MysteryDungeonRank,
    combat_stats: GenericCharacterStats,
    social_stats: GenericCharacterStats,
}

impl CharacterDataForStatEditing {
    pub fn remaining_combat_points(&self) -> i64 {
        helpers::calculate_available_combat_points(self.level)
            - self.combat_stats.calculate_invested_stat_points()
    }
    pub fn remaining_social_points(&self) -> i64 {
        helpers::calculate_available_social_points(&self.rank) as i64
            - self.social_stats.calculate_invested_stat_points()
    }
}

async fn get_character_data_for_edit(
    data: &Data,
    character_id: i64,
) -> CharacterDataForStatEditing {
    let record = sqlx::query!(
        "SELECT name, guild_id, experience, species_api_id, is_shiny, phenotype, \
                      stat_strength, stat_dexterity, stat_vitality, stat_special, stat_insight,
                      stat_edit_strength, stat_edit_dexterity, stat_edit_vitality, stat_edit_special, stat_edit_insight,
                      stat_tough, stat_cool, stat_beauty, stat_cute, stat_clever,
                      stat_edit_tough, stat_edit_cool, stat_edit_beauty, stat_edit_cute, stat_edit_clever
                FROM character WHERE id = ? \
                ORDER BY rowid \
                LIMIT 1",
        character_id,
    )
        .fetch_one(&data.database)
        .await
        .unwrap();

    let level = helpers::calculate_level_from_experience(record.experience);
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

    let pokemon_evolution_form_for_stats =
        helpers::get_usual_evolution_stage_for_level(level, pokemon, data);
    let combat_stats = GenericCharacterStats::from_combat_with_current_min(
        pokemon_evolution_form_for_stats,
        record.stat_edit_strength,
        record.stat_strength,
        record.stat_edit_dexterity,
        record.stat_dexterity,
        record.stat_edit_vitality,
        record.stat_vitality,
        record.stat_edit_special,
        record.stat_special,
        record.stat_edit_insight,
        record.stat_insight,
    );

    let social_stats = GenericCharacterStats::from_social_with_current_min(
        record.stat_edit_tough,
        record.stat_tough,
        record.stat_edit_cool,
        record.stat_cool,
        record.stat_edit_beauty,
        record.stat_beauty,
        record.stat_edit_cute,
        record.stat_cute,
        record.stat_edit_clever,
        record.stat_clever,
    );

    CharacterDataForStatEditing {
        name: record.name,
        id: character_id,
        emoji,
        level,
        rank,
        combat_stats,
        social_stats,
    }
}

struct MessageContent {
    pub content: String,
    pub ephemeral: bool,
    pub components: Vec<CreateActionRow>,
}

impl From<MessageContent> for CreateInteractionResponseMessage {
    fn from(value: MessageContent) -> Self {
        CreateInteractionResponseMessage::new()
            .content(value.content)
            .ephemeral(value.ephemeral)
            .components(value.components)
    }
}

impl From<MessageContent> for EditMessage {
    fn from(value: MessageContent) -> Self {
        EditMessage::new()
            .content(value.content)
            .components(value.components)
    }
}

impl From<MessageContent> for EditInteractionResponse {
    fn from(value: MessageContent) -> Self {
        EditInteractionResponse::new()
            .content(value.content)
            .components(value.components)
    }
}

async fn create_stat_edit_overview_message(
    data: &Data,
    character_id: i64,
    stat_type: StatType,
) -> MessageContent {
    let character_data = get_character_data_for_edit(data, character_id).await;

    let (stats, remaining_points) = match stat_type {
        StatType::Combat => {
            let combat_stats = &character_data.combat_stats;
            let remaining_points = character_data.remaining_combat_points();

            (combat_stats, remaining_points)
        }
        StatType::Social => {
            let social_stats = &character_data.social_stats;
            let remaining_points = character_data.remaining_social_points();

            (social_stats, remaining_points)
        }
    };

    let limit_break_substring = if stats.is_any_stat_at_or_above_max() {
        format!(
            "\nLimit breaking would cost you {}.",
            helpers::calculate_next_limit_break_cost(stats.count_limit_breaks())
        )
    } else {
        String::new()
    };

    let message = format!(
        "### {}{}\n```\n{}```\n{} Remaining Points.{}",
        character_data.emoji,
        character_data.name,
        stats.build_string(),
        remaining_points,
        limit_break_substring
    );

    MessageContent {
        content: message,
        ephemeral: true,
        components: match stat_type {
            StatType::Combat => create_combat_buttons(character_id),
            StatType::Social => create_social_buttons(character_id),
        },
    }
}

pub async fn update_character_post<'a>(ctx: &Context, data: &Data, id: i64) {
    if let Some(result) = crate::commands::characters::build_character_string(data, id).await {
        let message = ctx
            .http
            .get_message(
                ChannelId::from(result.stat_channel_id as u64),
                MessageId::from(result.stat_message_id as u64),
            )
            .await;
        if let Ok(mut message) = message {
            if let Err(e) = message
                .edit(
                    ctx,
                    EditMessage::new()
                        .content(&result.message)
                        .components(result.components.clone()),
                )
                .await
            {
                // Shouldn't happen since we just pressed a button in that thread and messages are ephemeral.
                // crate::commands::handle_error_during_message_edit(
                //     ctx,
                //     e,
                //     message,
                //     result.message,
                //     Some(result.components),
                //     result.name,
                // )
                // .await;
            }
        }
    }
}
