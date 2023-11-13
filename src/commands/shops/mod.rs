use crate::commands::shops::add_shop_owner::add_shop_owner;
use crate::commands::{BuildUpdatedStatMessageStringResult, Context};
use crate::data::Data;
use crate::{emoji, Error};
use poise::Command;

mod add_shop_owner;
mod initialize_shop;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![initialize_shop::initialize_shop(), add_shop_owner()]
}

pub async fn update_shop_post<'a>(ctx: &Context<'a>, shop_id: i64) {
    if let Some(result) = build_shop_string(ctx, shop_id).await {
        let message = ctx
            .serenity_context()
            .http
            .get_message(result.stat_channel_id as u64, result.stat_message_id as u64)
            .await;
        if let Ok(mut message) = message {
            match message.edit(ctx, |f| f.content(&result.message)).await {
                Ok(_) => {}
                Err(e) => {
                    let _ = ctx
                        .say(format!(
                            "**Failed to update the shop message for {}!**.\nThe change has been tracked, but whilst updating the message the following issue occurred: **{}**.\n\
                            In case this says 'Thread was archived', you can probably fix this by opening the forum post and then adding and removing one poke from the shop in order to trigger another update.",
                            result.name,
                            e
                        ))
                        .await;
                }
            }
        }
    }
}

async fn build_shop_string<'a>(
    ctx: &Context<'a>,
    shop_id: i64,
) -> Option<BuildUpdatedStatMessageStringResult> {
    let entry = sqlx::query!(
        "SELECT name, money, bot_message_id, bot_message_channel_id, creation_timestamp \
            FROM shop WHERE id = ? ORDER BY rowid LIMIT 1",
        shop_id
    )
    .fetch_one(&ctx.data().database)
    .await;

    match entry {
        Ok(entry) => Some(BuildUpdatedStatMessageStringResult {
            message: format!(
                "\
## {}
{} {}
",
                entry.name,
                entry.money,
                emoji::POKE_COIN,
            ),
            name: entry.name,
            stat_channel_id: entry.bot_message_channel_id,
            stat_message_id: entry.bot_message_id,
        }),
        Err(_) => None,
    }
}
