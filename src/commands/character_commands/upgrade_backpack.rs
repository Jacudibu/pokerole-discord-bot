use std::time::Duration;

use poise::{CreateReply, ReplyHandle};
use serenity::all::{ButtonStyle, CreateActionRow};

use crate::commands::autocompletion::autocomplete_owned_character_name;
use crate::commands::{Error, find_character, send_error};
use crate::shared::action_log::{ActionType, LogActionArguments};
use crate::shared::character::update_character_post_with_poise_context;
use crate::shared::utility::button_building;
use crate::shared::{PoiseContext, action_log, constants, emoji};

const CONFIRM: &str = "upgrade_backpack_proceed";
const ABORT: &str = "upgrade_backpack_abort";
const BASE_PRICE: i64 = 500;
const MONEY_PER_LEVEL: i64 = 500;

/// See what it takes to upgrade your backpack!
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, guild_only)]
pub async fn upgrade_backpack(
    ctx: PoiseContext<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_owned_character_name"]
    character: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;
    let character_record = sqlx::query!(
        "SELECT money, backpack_upgrade_count FROM character WHERE id = ?",
        character.id
    )
    .fetch_one(&ctx.data().database)
    .await?;

    let required_money = BASE_PRICE + MONEY_PER_LEVEL * character_record.backpack_upgrade_count;
    let target_slots =
        constants::DEFAULT_BACKPACK_SLOTS + character_record.backpack_upgrade_count + 1;
    if character_record.money < required_money {
        return send_error(
            &ctx,
            format!(
                "**Unable to upgrade {}'s backpack.**\n*Upgrading to {} slots would require {} {}. Right now, {} only owns {} {}.*",
                character.name,
                target_slots,
                required_money,
                emoji::POKE_COIN,
                character.name,
                character_record.money,
                emoji::POKE_COIN
            )
                .as_str(),
        )
            .await;
    }

    let original_message = format!(
        "**Upgrading {}'s backpack to {} slots will require {} {}.**",
        character.name,
        target_slots,
        required_money,
        emoji::POKE_COIN,
    );

    let reply = ctx
        .send(
            CreateReply::default()
                .content(original_message.clone())
                .components(vec![CreateActionRow::Buttons(vec![
                    button_building::create_styled_button(
                        "Let's do it!",
                        CONFIRM,
                        false,
                        ButtonStyle::Success,
                    ),
                    button_building::create_styled_button(
                        "Nope!",
                        ABORT,
                        false,
                        ButtonStyle::Danger,
                    ),
                ])]),
        )
        .await?;
    let message = reply.message().await?;

    let interaction = message
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(30))
        .await;

    if let Some(interaction) = interaction {
        if interaction.data.custom_id == CONFIRM {
            let updated_money = character_record.money - required_money;
            let updated_backpack_upgrade_count = character_record.backpack_upgrade_count + 1;

            let query_result = sqlx::query!(
                        "UPDATE character SET money = ?, backpack_upgrade_count = ? WHERE id = ? AND money = ? and backpack_upgrade_count = ?",
                        updated_money,
                        updated_backpack_upgrade_count,
                        character.id,
                        character_record.money,
                        character_record.backpack_upgrade_count,
                    )
                .execute(&ctx.data().database)
                .await;

            if query_result.is_ok() && query_result.unwrap().rows_affected() == 1 {
                action_log::log_action(
                    &ActionType::Payment,
                    LogActionArguments::triggered_by_user(&ctx),
                    format!(
                        "Removed {} {} from {}",
                        required_money,
                        emoji::POKE_COIN,
                        character.name,
                    )
                    .as_str(),
                )
                .await?;
                action_log::log_action(
                    &ActionType::BackpackUpgrade,
                    LogActionArguments::triggered_by_user(&ctx),
                    format!("Increased {}'s backpack size by 1", character.name).as_str(),
                )
                .await?;

                respond_to_success(ctx, reply, original_message).await?;
                update_character_post_with_poise_context(&ctx, character.id).await;
                return Ok(());
            }
        } else {
            return respond_to_cancellation(ctx, reply, original_message).await;
        }

        return respond_to_unexpected_behaviour(ctx, reply, original_message).await;
    }

    respond_to_timeout(ctx, reply, original_message).await
}

async fn respond_to_success<'a>(
    ctx: PoiseContext<'a>,
    reply: ReplyHandle<'a>,
    original_message: String,
) -> Result<(), Error> {
    edit_message_and_delete_buttons(ctx, reply, original_message + "\n\n**Upgrade successful!**")
        .await
}

async fn respond_to_cancellation<'a>(
    ctx: PoiseContext<'a>,
    reply: ReplyHandle<'a>,
    original_message: String,
) -> Result<(), Error> {
    edit_message_and_delete_buttons(
        ctx,
        reply,
        original_message + "\n\n**Request was cancelled.**",
    )
    .await
}

async fn respond_to_unexpected_behaviour<'a>(
    ctx: PoiseContext<'a>,
    reply: ReplyHandle<'a>,
    original_message: String,
) -> Result<(), Error> {
    edit_message_and_delete_buttons(
        ctx,
        reply,
        original_message + "\n\n**Something went wrong.**\n*This should only happen if you're actively trying to game the system... and if that's the case, thanks for trying, but... please stop? xD*")
        .await
}

async fn respond_to_timeout<'a>(
    ctx: PoiseContext<'a>,
    reply: ReplyHandle<'a>,
    original_message: String,
) -> Result<(), Error> {
    edit_message_and_delete_buttons(
        ctx,
        reply,
        original_message + "\n\n**Request timed out. Use the command again if needed.**",
    )
    .await
}

async fn edit_message_and_delete_buttons<'a>(
    ctx: PoiseContext<'a>,
    reply: ReplyHandle<'a>,
    message: String,
) -> Result<(), Error> {
    reply
        .edit(
            ctx,
            CreateReply::default()
                .content(message)
                .components(Vec::new()),
        )
        .await?;
    Ok(())
}
