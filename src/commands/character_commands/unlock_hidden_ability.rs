use std::time::Duration;

use poise::{CreateReply, ReplyHandle};
use serenity::all::{ButtonStyle, CreateActionRow};

use crate::commands::autocompletion::autocomplete_owned_character_name;
use crate::commands::character_commands::ActionType;
use crate::commands::{character_commands, find_character, update_character_post, Error};
use crate::shared::errors::ValidationError;
use crate::shared::helpers;
use crate::shared::{emoji, PoiseContext};

const CONFIRM: &str = "unlock_hidden_ability_proceed";
const ABORT: &str = "unlock_hidden_ability_abort";
const PRICE: i64 = 2000;

/// Unlock your hidden ability!
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, guild_only)]
pub async fn unlock_hidden_ability(
    ctx: PoiseContext<'_>,
    #[description = "Which character?"]
    #[autocomplete = "autocomplete_owned_character_name"]
    character: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;
    let character_record = sqlx::query!(
        "SELECT money, is_hidden_ability_unlocked FROM character WHERE id = ?",
        character.id
    )
    .fetch_one(&ctx.data().database)
    .await?;

    if character_record.is_hidden_ability_unlocked {
        return Err(Box::new(ValidationError::new(
            "Seems like this character already has their hidden ability unlocked!",
        )));
    }

    if character_record.money < PRICE {
        return Err(Box::new(ValidationError::new(
            &format!(
                "**Unable to unlock {}'s hidden ability.**\n*That would require {} {}. Right now, {} only owns {} {}.*",
                character.name,
                PRICE,
                emoji::POKE_COIN,
                character.name,
                character_record.money,
                emoji::POKE_COIN
            )
        )));
    }

    let original_message = format!(
        "**Unlocking {}'s hidden ability will require {} {}.**",
        character.name,
        PRICE,
        emoji::POKE_COIN,
    );

    let reply = ctx
        .send(
            CreateReply::default()
                .content(original_message.clone())
                .components(vec![CreateActionRow::Buttons(vec![
                    helpers::create_styled_button(
                        "Let's do it!",
                        CONFIRM,
                        false,
                        ButtonStyle::Success,
                    ),
                    helpers::create_styled_button("Nope!", ABORT, false, ButtonStyle::Danger),
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
            let updated_money = character_record.money - PRICE;

            let query_result = sqlx::query!(
                        "UPDATE character SET money = ?, is_hidden_ability_unlocked = true WHERE id = ? AND money = ?",
                        updated_money,
                        character.id,
                        character_record.money,
                    )
                .execute(&ctx.data().database)
                .await;

            if query_result.is_ok() && query_result.unwrap().rows_affected() == 1 {
                character_commands::log_action(
                    &ActionType::Payment,
                    &ctx,
                    format!(
                        "Removed {} {} from {}",
                        PRICE,
                        emoji::POKE_COIN,
                        character.name,
                    )
                    .as_str(),
                )
                .await?;
                character_commands::log_action(
                    &ActionType::HiddenAbilityUnlock,
                    &ctx,
                    format!("Unlocked {}'s hidden ability!", character.name).as_str(),
                )
                .await?;

                respond_to_success(ctx, reply, original_message).await?;
                update_character_post(&ctx, character.id).await;
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
    edit_message_and_delete_buttons(ctx, reply, original_message + "\n\n**Unlock successful!**")
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
