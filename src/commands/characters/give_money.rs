use crate::cache::CharacterCacheItem;
use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::autocompletion::autocomplete_owned_character_name;
use crate::commands::characters::{change_character_stat_after_validation, ActionType};
use crate::commands::{
    ensure_character_has_money, ensure_user_owns_character, find_character, Context, Error,
};
use crate::emoji;
use crate::errors::ValidationError;

async fn transfer_money_between_characters<'a>(
    ctx: &Context<'a>,
    giver: CharacterCacheItem,
    receiver: CharacterCacheItem,
    amount: i64,
) -> Result<(), Error> {
    ensure_user_owns_character(ctx.author(), &giver)?;
    if giver.id == receiver.id {
        return Err(Box::new(ValidationError::new(&format!(
            "*You successfully transferred {} {} from your left to your right hand. Ha. Ha.*",
            amount,
            emoji::POKE_COIN
        ))));
    }

    ensure_character_has_money(ctx, &giver, amount, "give").await?;

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
