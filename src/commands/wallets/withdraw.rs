use crate::cache::{CharacterCacheItem, WalletCacheItem};
use crate::commands::autocompletion::{autocomplete_character_name, autocomplete_wallet_name};
use crate::commands::characters::{change_character_stat_after_validation, ActionType};
use crate::commands::wallets::change_wallet_stat_after_validation;
use crate::commands::{
    ensure_user_owns_wallet_or_is_gm, ensure_wallet_has_money, find_character, find_wallet,
    Context, Error,
};
use crate::emoji;

async fn transfer_money_from_wallet_to_character<'a>(
    ctx: &Context<'a>,
    character: CharacterCacheItem,
    wallet: WalletCacheItem,
    amount: i64,
) -> Result<(), Error> {
    ensure_user_owns_wallet_or_is_gm(
        ctx.data(),
        ctx.author().id.get() as i64,
        ctx.author_member()
            .await
            .expect("author_member should be set within guild context."),
        &wallet,
    )
    .await?;
    ensure_wallet_has_money(ctx.data(), &wallet, amount, "pay").await?;

    // TODO: Potential flaw: Money gets transferred by someone else in between, this might not be detected. Figure out how to use sqlx transactions instead.
    // For now, it should be fine if we only subtract the money - people are way more likely to complain in that case. :'D
    if let Ok(_) = change_wallet_stat_after_validation(
        ctx,
        "money",
        &wallet,
        -amount,
        &ActionType::WalletWithdrawal,
    )
    .await
    {
        if let Ok(_) = change_character_stat_after_validation(
            ctx,
            "money",
            &character,
            amount,
            &ActionType::WalletWithdrawal,
        )
        .await
        {
            ctx.say(format!(
                "***{}** has withdrawn {} {} from **{}***!",
                character.name,
                amount,
                emoji::POKE_COIN,
                wallet.name
            ))
            .await?;
        } else {
            // TODO: The undo might fail.
            change_wallet_stat_after_validation(ctx, "money", &wallet, amount, &ActionType::Undo)
                .await?;
        }
    }

    Ok(())
}

/// Withdraw money from a wallet one of your character owns.
#[poise::command(slash_command, guild_only)]
pub async fn withdraw(
    ctx: Context<'_>,
    #[min = 1_u32] amount: u32,
    #[description = "What's the wallet's name?"]
    #[autocomplete = "autocomplete_wallet_name"]
    wallet: String,
    #[description = "To whom?"]
    #[autocomplete = "autocomplete_character_name"]
    character: String,
) -> Result<(), Error> {
    // TODO: Button to undo the transaction which lasts for a minute or so.
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let character = find_character(ctx.data(), guild_id, &character).await?;
    let wallet = find_wallet(ctx.data(), guild_id, &wallet).await?;

    transfer_money_from_wallet_to_character(&ctx, character, wallet, amount as i64).await
}
