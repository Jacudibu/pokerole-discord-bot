use crate::commands::autocompletion::{
    autocomplete_owned_character_name, autocomplete_wallet_name,
};
use crate::commands::character_commands::change_character_stat_after_validation;
use crate::commands::wallet_commands::change_wallet_stat_after_validation;
use crate::commands::{
    Error, ensure_character_has_money, ensure_user_owns_character, find_character, find_wallet,
};
use crate::shared::action_log::ActionType;
use crate::shared::cache::{CharacterCacheItem, WalletCacheItem};
use crate::shared::{PoiseContext, emoji};

async fn transfer_money_from_character_to_wallet<'a>(
    ctx: &PoiseContext<'a>,
    character: CharacterCacheItem,
    wallet: WalletCacheItem,
    amount: i64,
) -> Result<(), Error> {
    ensure_user_owns_character(ctx.author(), &character)?;
    ensure_character_has_money(ctx.data(), &character, amount, "pay").await?;

    // TODO: Potential flaw: Money gets transferred by someone else in between, this might not be detected. Figure out how to use sqlx transactions instead.
    // For now, it should be fine if we only subtract the money - people are way more likely to complain in that case. :'D
    if let Ok(_) = change_character_stat_after_validation(
        ctx,
        "money",
        &character,
        -amount,
        &ActionType::WalletPayment,
    )
    .await
    {
        if let Ok(_) = change_wallet_stat_after_validation(
            ctx,
            "money",
            &wallet,
            amount,
            &ActionType::WalletPayment,
        )
        .await
        {
            ctx.say(format!(
                "***{}** paid {} {} to **{}***!",
                character.name,
                amount,
                emoji::POKE_COIN,
                wallet.name
            ))
            .await?;
        } else {
            // TODO: The undo might fail.
            change_character_stat_after_validation(
                ctx,
                "money",
                &character,
                amount,
                &ActionType::Undo,
            )
            .await?;
        }
    }

    Ok(())
}

/// Pay money to a wallet.
#[poise::command(slash_command, guild_only)]
pub async fn pay(
    ctx: PoiseContext<'_>,
    #[description = "Who's paying?"]
    #[autocomplete = "autocomplete_owned_character_name"]
    character: String,
    #[min = 1_u32] amount: u32,
    #[description = "What's the wallet's name?"]
    #[autocomplete = "autocomplete_wallet_name"]
    wallet: String,
) -> Result<(), Error> {
    // TODO: Button to undo the transaction which lasts for a minute or so.
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;
    let wallet = find_wallet(ctx.data(), guild_id, &wallet).await?;

    transfer_money_from_character_to_wallet(&ctx, character, wallet, amount as i64).await
}
