use crate::character_stats::GenericCharacterStats;
use crate::data::Data;
use crate::enums::MysteryDungeonRank;
use crate::events::character_stat_edit::{
    create_stat_edit_overview_message, reset_stat_edit_values, StatType,
};
use crate::events::send_error;
use crate::game_data::PokemonApiId;
use crate::{helpers, Error};
use serenity::all::{ComponentInteraction, Context, CreateInteractionResponse};
use std::str::FromStr;

pub async fn initialize(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &Data,
    mut args: Vec<&str>,
) -> Result<(), Error> {
    match args.remove(0) {
        "combat" => initialize_combat(ctx, interaction, data, args).await,
        "social" => initialize_social(ctx, interaction, data, args).await,
        &_ => send_error(&interaction, ctx, "Are you trying to do anything cheesy?").await,
    }
}

async fn initialize_combat(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &Data,
    args: Vec<&str>,
) -> Result<(), Error> {
    let user_id = interaction.user.id.get() as i64;

    if let Some(character_id) = args.first() {
        let character_id = i64::from_str(character_id)?;
        let record = sqlx::query!(
            "SELECT experience, species_api_id, \
                      stat_strength, stat_dexterity, stat_vitality, stat_special, stat_insight
                FROM character WHERE id = ? AND user_id = ? \
                ORDER BY rowid \
                LIMIT 1",
            character_id,
            user_id
        )
        .fetch_one(&data.database)
        .await;

        return match record {
            Ok(record) => {
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

                let pokemon_evolution_form_for_stats =
                    helpers::get_usual_evolution_stage_for_level(level, pokemon, data);
                let combat_stats = GenericCharacterStats::from_combat(
                    pokemon_evolution_form_for_stats,
                    record.stat_strength,
                    record.stat_dexterity,
                    record.stat_vitality,
                    record.stat_special,
                    record.stat_insight,
                );

                let remaining_points = helpers::calculate_available_combat_points(level)
                    - combat_stats.calculate_invested_stat_points();

                if remaining_points <= 0 {
                    return send_error(
                        &interaction,
                        ctx,
                        "This character doesn't seem to have any remaining combat stat points.",
                    )
                    .await;
                }

                reset_stat_edit_values(data, character_id).await;
                let _ = interaction
                    .create_response(
                        ctx,
                        CreateInteractionResponse::Message(
                            create_stat_edit_overview_message(data, character_id, StatType::Combat)
                                .await
                                .into(),
                        ),
                    )
                    .await;

                Ok(())
            }
            _ => {
                send_error(
                    &interaction,
                    ctx,
                    "You don't seem to own this character. No touchies! *hiss*",
                )
                .await
            }
        };
    }

    send_error(&interaction, ctx, "Are you trying to do anything cheesy?").await
}
async fn initialize_social(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &Data,
    args: Vec<&str>,
) -> Result<(), Error> {
    let user_id = interaction.user.id.get() as i64;

    if let Some(character_id) = args.first() {
        let character_id = i64::from_str(character_id)?;
        let record = sqlx::query!(
            "SELECT experience, species_api_id, \
                    stat_tough, stat_cool, stat_beauty, stat_cute, stat_clever
                FROM character WHERE id = ? AND user_id = ? \
                ORDER BY rowid \
                LIMIT 1",
            character_id,
            user_id
        )
        .fetch_one(&data.database)
        .await;

        return match record {
            Ok(record) => {
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

                let pokemon_evolution_form_for_stats =
                    helpers::get_usual_evolution_stage_for_level(level, pokemon, data);
                let social_stats = GenericCharacterStats::from_social(
                    record.stat_tough,
                    record.stat_cool,
                    record.stat_beauty,
                    record.stat_cute,
                    record.stat_clever,
                );

                let remaining_points = helpers::calculate_available_social_points(&rank) as i64
                    - social_stats.calculate_invested_stat_points();

                if remaining_points <= 0 {
                    return send_error(
                        &interaction,
                        ctx,
                        "This character doesn't seem to have any remaining social stat points.",
                    )
                    .await;
                }

                reset_stat_edit_values(data, character_id).await;
                let _ = interaction
                    .create_response(
                        ctx,
                        CreateInteractionResponse::Message(
                            create_stat_edit_overview_message(data, character_id, StatType::Social)
                                .await
                                .into(),
                        ),
                    )
                    .await;

                Ok(())
            }
            _ => {
                send_error(
                    &interaction,
                    ctx,
                    "You don't seem to own this character. No touchies! *hiss*",
                )
                .await
            }
        };
    }

    send_error(&interaction, ctx, "Are you trying to do anything cheesy?").await
}
