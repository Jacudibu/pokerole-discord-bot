use crate::character_stats::CharacterCombatStats;
use crate::data::Data;
use crate::events::character_stat_edit::{
    create_stat_edit_overview_message, get_character_data_for_edit, CharacterDataForStatEditing,
    StatType,
};
use crate::events::send_error;
use crate::Error;
use serenity::all::{ComponentInteraction, Context, EditInteractionResponse};
use std::str::FromStr;

pub async fn handle_combat_stat_request(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &Data,
    mut args: Vec<&str>,
) -> Result<(), Error> {
    match args.remove(0) {
        "add" => edit_combat_stat(ctx, interaction, data, args, 1).await,
        "subtract" => edit_combat_stat(ctx, interaction, data, args, -1).await,
        "apply" => apply_combat_stats(ctx, interaction, data, args).await,
        "cancel" => cancel_combat_stats(ctx, interaction).await,
        &_ => send_error(&interaction, ctx, "Are you trying to do anything cheesy?").await,
    }
}

async fn apply_combat_stats(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &Data,
    mut args: Vec<&str>,
) -> Result<(), Error> {
    let deferred_interaction = interaction.defer(ctx);
    let character_id = i64::from_str(args.remove(0))?;

    let _ = sqlx::query!(
        "UPDATE character
SET
    stat_strength = stat_edit_strength,
    stat_dexterity = stat_edit_dexterity,
    stat_vitality = stat_edit_vitality,
    stat_special = stat_edit_special,
    stat_insight = stat_edit_insight
WHERE id = ?",
        character_id
    )
    .execute(&data.database)
    .await;

    let _ = deferred_interaction.await;
    let _ = interaction
        .edit_response(
            ctx,
            EditInteractionResponse::new()
                .content("Successfully applied your stats.")
                .components(Vec::new()),
        )
        .await;

    Ok(())
}

async fn cancel_combat_stats(
    ctx: &Context,
    interaction: &ComponentInteraction,
) -> Result<(), Error> {
    let _ = interaction.defer(ctx).await;
    let _ = interaction
        .edit_response(
            ctx,
            EditInteractionResponse::new()
                .content("Operation cancelled.")
                .components(Vec::new()),
        )
        .await;

    Ok(())
}

#[rustfmt::skip]
async fn edit_combat_stat(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &Data,
    mut args: Vec<&str>,
    amount: i64,
) -> Result<(), Error> {
    let character_id = i64::from_str(args.remove(0))?;
    let character = get_character_data_for_edit(data, character_id).await;

    match args.remove(0) {
        "strength" => edit_combat_stat_bla(ctx, interaction, data, character, amount, CharacterCombatStats::Strength).await,
        "dexterity" => edit_combat_stat_bla(ctx, interaction, data, character, amount, CharacterCombatStats::Dexterity).await,
        "vitality" => edit_combat_stat_bla(ctx, interaction, data, character, amount, CharacterCombatStats::Vitality).await,
        "special" => edit_combat_stat_bla(ctx, interaction, data, character, amount, CharacterCombatStats::Special).await,
        "insight" => edit_combat_stat_bla(ctx, interaction, data, character, amount, CharacterCombatStats::Insight).await,
        &_ => send_error(&interaction, ctx, "Are you trying to do anything cheesy?").await,
    }
}

async fn edit_combat_stat_bla(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &Data,
    character: CharacterDataForStatEditing,
    amount: i64,
    stat: CharacterCombatStats,
) -> Result<(), Error> {
    let deferred_interaction = interaction.defer(ctx);
    let edited_stat = character.combat_stats.get_combat(stat);
    if amount > 0 {
        let remaining_points = character.remaining_combat_points();
        if remaining_points == 0 {
            return send_error(
                &interaction,
                ctx,
                "Seems like you don't have any remaining stat points!",
            )
            .await;
        }

        let points_required_for_limit_break = 2 + character.combat_stats.count_limit_breaks();
        if edited_stat.current + 1 > edited_stat.max
            && remaining_points < points_required_for_limit_break
        {
            return send_error(
                &interaction,
                ctx,
                &format!(
                    "Cannot apply limit break: You'd need {} points, but only got {}.",
                    points_required_for_limit_break, remaining_points
                ),
            )
            .await;
        }
    } else if edited_stat.current == edited_stat.min {
        return send_error(
            &interaction,
            ctx,
            &format!("Unable to reduce your {:?} any further.", stat),
        )
        .await;
    }

    let _ = sqlx::query(&format!(
        "UPDATE character SET {} = ? WHERE id = ?",
        get_edit_string(stat)
    ))
    .bind(edited_stat.current + amount)
    .bind(character.id)
    .execute(&data.database)
    .await;

    let edit_message =
        create_stat_edit_overview_message(data, character.id, StatType::Combat).await;

    let _ = deferred_interaction.await;
    let _ = interaction.edit_response(ctx, edit_message.into()).await;

    Ok(())
}

fn get_edit_string(stat: CharacterCombatStats) -> String {
    match stat {
        CharacterCombatStats::Strength => String::from("stat_edit_strength"),
        CharacterCombatStats::Dexterity => String::from("stat_edit_dexterity"),
        CharacterCombatStats::Vitality => String::from("stat_edit_vitality"),
        CharacterCombatStats::Special => String::from("stat_edit_special"),
        CharacterCombatStats::Insight => String::from("stat_edit_insight"),
    }
}
