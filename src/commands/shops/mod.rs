use crate::cache::ShopCacheItem;
use crate::commands::characters::{log_action, ActionType, EntityWithNameAndNumericValue};
use crate::commands::{send_error, BuildUpdatedStatMessageStringResult, Context};
use crate::data::Data;
use crate::{emoji, Error};
use poise::Command;

mod add_shop_owner;
mod initialize_shop;
mod pay;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![
        initialize_shop::initialize_shop(),
        add_shop_owner::add_shop_owner(),
        pay::pay(),
    ]
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

    let owners = sqlx::query!(
        "SELECT character.name FROM character WHERE id in (\
                SELECT character_id FROM shop_owner WHERE shop_id = ?)",
        shop_id
    )
    .fetch_all(&ctx.data().database)
    .await;

    let owner_line;
    if let Ok(owners) = owners {
        if owners.is_empty() {
            owner_line = String::new()
        } else if owners.len() == 1 {
            owner_line = format!("\n**Owner**: {}", owners.get(0).expect("len = 1").name);
        } else {
            owner_line = format!(
                "\n**Owners**: {}",
                owners
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        }
    } else {
        owner_line = String::new()
    }

    match entry {
        Ok(entry) => Some(BuildUpdatedStatMessageStringResult {
            message: format!(
                "\
## {}{}
{} {}
",
                entry.name,
                owner_line,
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

pub async fn change_shop_stat_after_validation<'a>(
    ctx: &Context<'a>,
    database_column: &str,
    shop: &ShopCacheItem,
    amount: i64,
    action_type: &ActionType,
) -> Result<(), Error> {
    ctx.defer().await?;
    let record = sqlx::query_as::<_, EntityWithNameAndNumericValue>(
        format!(
            "SELECT id, name, {} as value FROM shop WHERE id = ?",
            database_column
        )
        .as_str(),
    )
    .bind(shop.id)
    .fetch_one(&ctx.data().database)
    .await;

    match record {
        Ok(record) => {
            let new_value = record.value + amount;
            let result = sqlx::query(
                format!("UPDATE shop SET {} = ? WHERE id = ? AND {} = ?", database_column, database_column).as_str())
                .bind(new_value)
                .bind(record.id)
                .bind(record.value)
                .execute(&ctx.data().database).await;

            if result.is_err() || result.unwrap().rows_affected() != 1 {
                return crate::commands::characters::send_stale_data_error(ctx).await
            }

            update_shop_post(ctx, record.id).await;
            let action = if database_column == "money" {
                emoji::POKE_COIN
            } else {
                database_column
            };
            let added_or_removed: &str;
            let to_or_from: &str;
            if amount > 0 {
                added_or_removed = "Added";
                to_or_from = "to";
            } else {
                added_or_removed = "Removed";
                to_or_from = "from";
            }

            log_action(action_type, ctx, format!("{} {} {} {} {}", added_or_removed, amount.abs(), action, to_or_from, record.name).as_str()).await
        }
        Err(_) => {
            send_error(ctx, format!("Unable to find a shop named {}.\n**Internal cache must be out of date. Please let me know if this ever happens.**", shop.name).as_str()).await
        }
    }
}
