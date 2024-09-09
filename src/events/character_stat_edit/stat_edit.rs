use std::str::FromStr;

use crate::character_stats::SingleCharacterStatType;
use crate::events::character_stat_edit::{
    create_stat_edit_overview_message, get_character_data_for_edit, CharacterDataForStatEditing,
    StatType,
};
use crate::events::{send_error, update_character_post};
use crate::game_data::GameData;
use crate::{helpers, Error};
use serenity::all::{ComponentInteraction, Context, EditInteractionResponse};
use sqlx::{Pool, Sqlite};

#[rustfmt::skip]
pub async fn handle_edit_stat_request(
    ctx: &Context,
    interaction: &ComponentInteraction,
    database: &Pool<Sqlite>,
    game_data: &GameData,
    mut args: Vec<&str>,
) -> Result<(), Error> {
    let character_id = i64::from_str(args.remove(0))?;

    match args.remove(0) {
        "add" => edit_stat(ctx, interaction, database, game_data, character_id, args, 1).await,
        "subtract" => edit_stat(ctx, interaction, database, game_data, character_id, args, -1).await,
        "apply-combat" => apply_combat_stats(ctx, interaction, database, game_data, character_id).await,
        "apply-social" => apply_social_stats(ctx, interaction, database, game_data, character_id).await,
        "cancel" => cancel(ctx, interaction).await,
        &_ => send_error(&interaction, ctx, "Are you trying to do anything cheesy?").await,
    }
}

async fn apply_combat_stats(
    ctx: &Context,
    interaction: &ComponentInteraction,
    database: &Pool<Sqlite>,
    game_data: &GameData,
    character_id: i64,
) -> Result<(), Error> {
    let deferred_interaction = interaction.defer(ctx);

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
    .execute(database)
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

    update_character_post(ctx, database, game_data, character_id).await;
    Ok(())
}

async fn apply_social_stats(
    ctx: &Context,
    interaction: &ComponentInteraction,
    database: &Pool<Sqlite>,
    game_data: &GameData,
    character_id: i64,
) -> Result<(), Error> {
    let deferred_interaction = interaction.defer(ctx);
    let _ = sqlx::query!(
        "UPDATE character
SET
    stat_tough = stat_edit_tough,
    stat_cool = stat_edit_cool,
    stat_beauty = stat_edit_beauty,
    stat_cute = stat_edit_cute,
    stat_clever = stat_edit_clever
WHERE id = ?",
        character_id
    )
    .execute(database)
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

    update_character_post(ctx, database, game_data, character_id).await;
    Ok(())
}

async fn cancel(ctx: &Context, interaction: &ComponentInteraction) -> Result<(), Error> {
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
async fn edit_stat(
    ctx: &Context,
    interaction: &ComponentInteraction,
    database: &Pool<Sqlite>,
    game_data: &GameData,
    character_id: i64,
    mut args: Vec<&str>,
    amount: i64,
) -> Result<(), Error> {
    let character = get_character_data_for_edit(database, game_data, character_id).await;

    match args.remove(0) {
        "strength" => edit_specific_stat(ctx, interaction, database, game_data, character, amount, SingleCharacterStatType::Strength).await,
        "dexterity" => edit_specific_stat(ctx, interaction, database, game_data, character, amount, SingleCharacterStatType::Dexterity).await,
        "vitality" => edit_specific_stat(ctx, interaction, database, game_data, character, amount, SingleCharacterStatType::Vitality).await,
        "special" => edit_specific_stat(ctx, interaction, database, game_data, character, amount, SingleCharacterStatType::Special).await,
        "insight" => edit_specific_stat(ctx, interaction, database, game_data, character, amount, SingleCharacterStatType::Insight).await,
        "tough" => edit_specific_stat(ctx, interaction, database, game_data, character, amount, SingleCharacterStatType::Tough).await,
        "cool" => edit_specific_stat(ctx, interaction, database, game_data, character, amount, SingleCharacterStatType::Cool).await,
        "beauty" => edit_specific_stat(ctx, interaction, database, game_data, character, amount, SingleCharacterStatType::Beauty).await,
        "cute" => edit_specific_stat(ctx, interaction, database, game_data, character, amount, SingleCharacterStatType::Cute).await,
        "clever" => edit_specific_stat(ctx, interaction, database, game_data, character, amount, SingleCharacterStatType::Clever).await,
        &_ => send_error(&interaction, ctx, "Are you trying to do anything cheesy?").await,
    }
}

async fn edit_specific_stat(
    ctx: &Context,
    interaction: &ComponentInteraction,
    database: &Pool<Sqlite>,
    game_data: &GameData,
    character: CharacterDataForStatEditing,
    amount: i64,
    stat: SingleCharacterStatType,
) -> Result<(), Error> {
    let deferred_interaction = interaction.defer(ctx);
    let edited_stat = if stat.is_combat_stat() {
        character.combat_stats.get(stat)
    } else {
        character.social_stats.get(stat)
    };

    if amount > 0 {
        let (remaining_points, limit_break_count) = if stat.is_combat_stat() {
            (
                character.remaining_combat_points(),
                character.combat_stats.count_limit_breaks(),
            )
        } else {
            (
                character.remaining_social_points(),
                character.social_stats.count_limit_breaks(),
            )
        };
        if remaining_points == 0 {
            return send_error(
                &interaction,
                ctx,
                "Seems like you don't have any remaining stat points!",
            )
            .await;
        }

        let points_required_for_limit_break =
            helpers::calculate_next_limit_break_cost(limit_break_count);
        if edited_stat.current + 1 > edited_stat.species_max
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
    } else if edited_stat.current == edited_stat.currently_set_on_character {
        return if edited_stat.current == edited_stat.species_min {
            send_error(
                &interaction,
                ctx,
                &format!("Unable to reduce your {:?} any further. That's the lowest your species can go.", stat),
            ).await
        } else {
            send_error(
                &interaction,
                ctx,
                &format!("Unable to reduce your {:?} any further. You cannot remove stat points which you've previously assigned.", stat),
            ).await
        };
    }

    let _ = sqlx::query(&format!(
        "UPDATE character SET {} = ? WHERE id = ?",
        get_edit_string(stat)
    ))
    .bind(edited_stat.current + amount)
    .bind(character.id)
    .execute(database)
    .await;

    let edit_message = create_stat_edit_overview_message(
        database,
        game_data,
        character.id,
        if stat.is_combat_stat() {
            StatType::Combat
        } else {
            StatType::Social
        },
    )
    .await;

    let _ = deferred_interaction.await;
    let _ = interaction.edit_response(ctx, edit_message.into()).await;

    Ok(())
}

fn get_edit_string(stat: SingleCharacterStatType) -> String {
    match stat {
        SingleCharacterStatType::Strength => String::from("stat_edit_strength"),
        SingleCharacterStatType::Dexterity => String::from("stat_edit_dexterity"),
        SingleCharacterStatType::Vitality => String::from("stat_edit_vitality"),
        SingleCharacterStatType::Special => String::from("stat_edit_special"),
        SingleCharacterStatType::Insight => String::from("stat_edit_insight"),
        SingleCharacterStatType::Tough => String::from("stat_edit_tough"),
        SingleCharacterStatType::Cool => String::from("stat_edit_cool"),
        SingleCharacterStatType::Beauty => String::from("stat_edit_beauty"),
        SingleCharacterStatType::Cute => String::from("stat_edit_cute"),
        SingleCharacterStatType::Clever => String::from("stat_edit_clever"),
    }
}
