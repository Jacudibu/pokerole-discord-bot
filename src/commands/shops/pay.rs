use crate::cache::{CharacterCacheItem, ShopCacheItem};
use crate::commands::autocompletion::{autocomplete_owned_character_name, autocomplete_shop_name};
use crate::commands::characters::{change_character_stat_after_validation, ActionType};
use crate::commands::shops::change_shop_stat_after_validation;
use crate::commands::{find_character, find_shop, send_error, Context, Error};
use crate::emoji;
use crate::parse_error::ParseError;
use serenity::model::user::User;

async fn transfer_money_from_character_to_shop<'a>(
    ctx: &Context<'a>,
    character: CharacterCacheItem,
    shop: ShopCacheItem,
    amount: i64,
) -> Result<(), Error> {
    // TODO: verify this sends an ephemeral response and make give_money also use this
    verify_user_owns_character(ctx.author(), &character)?;

    let character_record = sqlx::query!("SELECT money FROM character WHERE id = ?", character.id)
        .fetch_one(&ctx.data().database)
        .await;

    if let Ok(character_record) = character_record {
        // TODO: Extract like verify_user_owns_character
        if character_record.money < amount {
            return send_error(
                ctx,
                format!(
                    "**Unable to pay {} {}.**\n*{} only owns {} {}.*",
                    amount,
                    emoji::POKE_COIN,
                    character.name,
                    character_record.money,
                    emoji::POKE_COIN
                )
                .as_str(),
            )
            .await;
        }
    } else {
        return send_error(ctx, format!("**Something went wrong when checking how much money {} has. Please try again. Let me know if this ever happens.**",
                                       character.name).as_str()
        ).await;
    }

    // TODO: Potential flaw: Money gets transferred by someone else in between, this might not be detected. Figure out how to use sqlx transactions instead.
    // For now, it should be fine if we only subtract the money - people are way more likely to complain in that case. :'D
    if let Ok(_) = change_character_stat_after_validation(
        ctx,
        "money",
        &character,
        -amount,
        &ActionType::TradeOutgoing,
    )
    .await
    {
        if let Ok(_) = change_shop_stat_after_validation(
            ctx,
            "money",
            &shop,
            amount,
            &ActionType::TradeIncoming,
        )
        .await
        {
            ctx.say(format!(
                "***{}** paid {} {} to **{}***!",
                character.name,
                amount,
                emoji::POKE_COIN,
                shop.name
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

fn verify_user_owns_character(user: &User, giver: &CharacterCacheItem) -> Result<(), ParseError> {
    if giver.user_id == user.id.0 {
        Ok(())
    } else {
        Err(ParseError::new(&format!(
            "You don't seem to own a character named {} on this server.",
            giver.name
        )))
    }
}

/// Pay money to a shop.
#[poise::command(slash_command, guild_only)]
pub async fn pay(
    ctx: Context<'_>,
    #[description = "Who's paying?"]
    #[autocomplete = "autocomplete_owned_character_name"]
    character: String,
    #[min = 1_u32] amount: u32,
    #[description = "What's the shop's name?"]
    #[autocomplete = "autocomplete_shop_name"]
    shop: String,
) -> Result<(), Error> {
    // TODO: Button to undo the transaction which lasts for a minute or so.
    let guild_id = ctx.guild_id().expect("Command is guild_only").0;
    let character = find_character(ctx.data(), guild_id, &character).await?;
    let shop = find_shop(ctx.data(), guild_id, &shop).await?;

    transfer_money_from_character_to_shop(&ctx, character, shop, amount as i64).await
}
