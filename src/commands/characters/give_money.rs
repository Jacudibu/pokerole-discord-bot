use crate::emoji;
use crate::commands::{Context, Error, send_error};
use crate::commands::characters::change_character_stat;
use crate::commands::autocompletion::autocomplete_character_name;
use crate::commands::autocompletion::autocomplete_owned_character_name;

/// Transfer money between characters.
#[poise::command(slash_command, guild_only)]
pub async fn give_money(
    ctx: Context<'_>,
    #[description = "Who gives the money?"]
    #[autocomplete = "autocomplete_owned_character_name"]
    giver: String,
    amount: u32,
    #[description = "To whom?"]
    #[autocomplete = "autocomplete_character_name"]
    receiver: String,
) -> Result<(), Error> {
    // TODO: Button to undo the transaction which lasts for a minute or so.
    let amount = amount as i64;
    let user_id = ctx.author().id.0 as i64;
    let guild_id = ctx.guild_id().expect("Command is guild_only").0 as i64;

    let sender_record = sqlx::query!("SELECT id, money FROM character WHERE name = ? AND guild_id = ? AND user_id = ?",
        giver,
        guild_id,
        user_id
    ).fetch_one(&ctx.data().database)
        .await;

    if let Ok(sender_record) = sender_record {
        if sender_record.money < amount {
            return send_error(&ctx, format!("**Unable to send {}.**\n*You only own {} {}.*",
                                            amount, sender_record.money, emoji::POKE_COIN).as_str()
            ).await;
        }

        let receiver_record = sqlx::query!("SELECT id FROM character WHERE name = ? AND guild_id = ?",
            receiver,
            guild_id,
        ).fetch_one(&ctx.data().database)
            .await;

        if let Ok(receiver_record) = receiver_record {
            if receiver_record.id == sender_record.id {
                return send_error(&ctx, format!("*You successfully transferred {} {} from your left to your right hand. Ha. Ha.*",
                                                amount, emoji::POKE_COIN).as_str()
                ).await;
            }
        } else {
            return send_error(&ctx, format!("Unable to find a character named {}.", receiver).as_str()).await;
        }
    } else {
        return send_error(&ctx, format!("You don't seem to own a character named {} on this server.", giver).as_str()
        ).await;
    }

    // TODO: Potential flaw: Money gets transferred by someone else in between this might not be detected.
    if let Ok(_) = change_character_stat(&ctx, "money", &giver, -amount).await {
        if let Ok(_) = change_character_stat(&ctx, "money", &receiver, amount).await {
            ctx.say(format!("{} gave {} {} to {}!", giver, amount, emoji::POKE_COIN, receiver)).await?;
        } else {
            // TODO: The undo might fail.
            change_character_stat(&ctx, "money", &giver, amount).await?;
        }
    }

    Ok(())
}
