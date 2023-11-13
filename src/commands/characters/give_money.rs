use crate::cache::CharacterCacheItem;
use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::autocompletion::autocomplete_owned_character_name;
use crate::commands::characters::{change_character_stat_after_validation, ActionType};
use crate::commands::{find_character, send_error, Context, Error};
use crate::emoji;

async fn transfer_money_between_characters<'a>(
    ctx: &Context<'a>,
    giver: CharacterCacheItem,
    receiver: CharacterCacheItem,
    amount: i64,
) -> Result<(), Error> {
    if giver.user_id != ctx.author().id.0 {
        return send_error(
            ctx,
            &format!(
                "You don't seem to own a character named {} on this server.",
                giver.name
            ),
        )
        .await;
    }

    if giver.id == receiver.id {
        return send_error(
            ctx,
            &format!(
                "*You successfully transferred {} {} from your left to your right hand. Ha. Ha.*",
                amount,
                emoji::POKE_COIN
            ),
        )
        .await;
    }

    let giver_record = sqlx::query!("SELECT money FROM character WHERE id = ?", giver.id)
        .fetch_one(&ctx.data().database)
        .await;

    if let Ok(giver_record) = giver_record {
        if giver_record.money < amount {
            return send_error(
                ctx,
                format!(
                    "**Unable to send {} {}.**\n*{} only owns {} {}.*",
                    amount,
                    emoji::POKE_COIN,
                    giver.name,
                    giver_record.money,
                    emoji::POKE_COIN
                )
                .as_str(),
            )
            .await;
        }
    } else {
        return send_error(ctx, format!("**Something went wrong when checking how much money {} has. Please try again. Let me know if this ever happens.**",
                                        giver.name).as_str()
        ).await;
    }

    // TODO: Potential flaw: Money gets transferred by someone else in between this might not be detected.
    // For now, it should be fine if we only subtract the money - people are way more likely to complain in that case. :'D
    if let Ok(_) = change_character_stat_after_validation(
        ctx,
        "money",
        &giver,
        -amount,
        &ActionType::TradeOutgoing,
    )
    .await
    {
        if let Ok(_) = change_character_stat_after_validation(
            ctx,
            "money",
            &receiver,
            amount,
            &ActionType::TradeIncoming,
        )
        .await
        {
            ctx.say(format!(
                "***{}** gave {} {} to **{}***!",
                giver.name,
                amount,
                emoji::POKE_COIN,
                receiver.name
            ))
            .await?;
        } else {
            // TODO: The undo might fail.
            change_character_stat_after_validation(ctx, "money", &giver, amount, &ActionType::Undo)
                .await?;
        }
    }

    Ok(())
}

/// Transfer money between characters.
#[poise::command(slash_command, guild_only)]
pub async fn give_money(
    ctx: Context<'_>,
    #[description = "Who gives the money?"]
    #[autocomplete = "autocomplete_owned_character_name"]
    giver: String,
    #[min = 1_u32] amount: u32,
    #[description = "To whom?"]
    #[autocomplete = "autocomplete_character_name"]
    receiver: String,
) -> Result<(), Error> {
    // TODO: Button to undo the transaction which lasts for a minute or so.
    let guild_id = ctx.guild_id().expect("Command is guild_only").0;
    let giver = find_character(ctx.data(), guild_id, &giver).await?;
    let receiver = find_character(ctx.data(), guild_id, &receiver).await?;

    transfer_money_between_characters(&ctx, giver, receiver, amount as i64).await
}
