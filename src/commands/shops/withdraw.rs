use crate::cache::{CharacterCacheItem, ShopCacheItem};
use crate::commands::autocompletion::{autocomplete_character_name, autocomplete_shop_name};
use crate::commands::characters::{change_character_stat_after_validation, ActionType};
use crate::commands::shops::change_shop_stat_after_validation;
use crate::commands::{
    ensure_shop_has_money, ensure_user_owns_shop_or_is_gm, find_character, find_shop, Context,
    Error,
};
use crate::emoji;

async fn transfer_money_from_shop_to_character<'a>(
    ctx: &Context<'a>,
    character: CharacterCacheItem,
    shop: ShopCacheItem,
    amount: i64,
) -> Result<(), Error> {
    ensure_user_owns_shop_or_is_gm(
        ctx.data(),
        ctx.author().id.0 as i64,
        ctx.author_member()
            .await
            .expect("author_member should be set within guild context."),
        &shop,
    )
    .await?;
    ensure_shop_has_money(ctx.data(), &shop, amount, "pay").await?;

    // TODO: Potential flaw: Money gets transferred by someone else in between, this might not be detected. Figure out how to use sqlx transactions instead.
    // For now, it should be fine if we only subtract the money - people are way more likely to complain in that case. :'D
    if let Ok(_) =
        change_shop_stat_after_validation(ctx, "money", &shop, -amount, &ActionType::ShopWithdrawal)
            .await
    {
        if let Ok(_) = change_character_stat_after_validation(
            ctx,
            "money",
            &character,
            amount,
            &ActionType::ShopWithdrawal,
        )
        .await
        {
            ctx.say(format!(
                "***{}** has withdrawn {} {} from **{}***!",
                character.name,
                amount,
                emoji::POKE_COIN,
                shop.name
            ))
            .await?;
        } else {
            // TODO: The undo might fail.
            change_shop_stat_after_validation(ctx, "money", &shop, amount, &ActionType::Undo)
                .await?;
        }
    }

    Ok(())
}

/// Withdraw money from a shop one of your character owns.
#[poise::command(slash_command, guild_only)]
pub async fn withdraw(
    ctx: Context<'_>,
    #[min = 1_u32] amount: u32,
    #[description = "What's the shop's name?"]
    #[autocomplete = "autocomplete_shop_name"]
    shop: String,
    #[description = "To whom?"]
    #[autocomplete = "autocomplete_character_name"]
    character: String,
) -> Result<(), Error> {
    // TODO: Button to undo the transaction which lasts for a minute or so.
    let guild_id = ctx.guild_id().expect("Command is guild_only").0;
    let character = find_character(ctx.data(), guild_id, &character).await?;
    let shop = find_shop(ctx.data(), guild_id, &shop).await?;

    transfer_money_from_shop_to_character(&ctx, character, shop, amount as i64).await
}
