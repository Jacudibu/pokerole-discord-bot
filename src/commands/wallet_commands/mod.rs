use crate::commands::character_commands::{log_action, ActionType, EntityWithNameAndNumericValue};
use crate::commands::{
    handle_error_during_message_edit, send_error, BuildUpdatedStatMessageStringResult,
};
use crate::shared::cache::WalletCacheItem;
use crate::shared::data::Data;
use crate::shared::{emoji, PoiseContext};
use crate::Error;
use poise::Command;
use serenity::all::{ChannelId, EditMessage, MessageId};

mod add_wallet_owner;
mod create_wallet;
mod edit_wallet;
mod pay;
mod wallet_info;
mod withdraw;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![
        create_wallet::create_wallet(),
        add_wallet_owner::add_wallet_owner(),
        pay::pay(),
        withdraw::withdraw(),
        wallet_info::wallet_info(),
        edit_wallet::edit_wallet(),
    ]
}

pub async fn update_wallet_post<'a>(ctx: &PoiseContext<'a>, wallet_id: i64) {
    if let Some(result) = build_wallet_string(ctx, wallet_id).await {
        let message = ctx
            .serenity_context()
            .http
            .get_message(
                ChannelId::from(result.stat_channel_id as u64),
                MessageId::from(result.stat_message_id as u64),
            )
            .await;
        if let Ok(mut message) = message {
            if let Err(e) = message
                .edit(ctx, EditMessage::new().content(&result.message))
                .await
            {
                handle_error_during_message_edit(
                    ctx,
                    e,
                    message,
                    result.message,
                    None,
                    result.name,
                )
                .await;
            }
        }
    }
}

async fn build_wallet_string<'a>(
    ctx: &PoiseContext<'a>,
    wallet_id: i64,
) -> Option<BuildUpdatedStatMessageStringResult> {
    let entry = sqlx::query!(
        "SELECT name, money, bot_message_id, bot_message_channel_id, creation_timestamp \
            FROM wallet WHERE id = ? ORDER BY rowid LIMIT 1",
        wallet_id
    )
    .fetch_one(&ctx.data().database)
    .await;

    let owners = sqlx::query!(
        "SELECT character.name FROM character WHERE id in (\
                SELECT character_id FROM wallet_owner WHERE wallet_id = ?)",
        wallet_id
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
## 👛 {}{}
{} {}
",
                entry.name,
                owner_line,
                entry.money,
                emoji::POKE_COIN,
            ),
            name: entry.name,
            components: Vec::new(),
            stat_channel_id: entry.bot_message_channel_id,
            stat_message_id: entry.bot_message_id,
        }),
        Err(_) => None,
    }
}

pub async fn change_wallet_stat_after_validation<'a>(
    ctx: &PoiseContext<'a>,
    database_column: &str,
    wallet: &WalletCacheItem,
    amount: i64,
    action_type: &ActionType,
) -> Result<(), Error> {
    ctx.defer().await?;
    let record = sqlx::query_as::<_, EntityWithNameAndNumericValue>(
        format!(
            "SELECT id, name, {} as value FROM wallet WHERE id = ?",
            database_column
        )
        .as_str(),
    )
    .bind(wallet.id)
    .fetch_one(&ctx.data().database)
    .await;

    match record {
        Ok(record) => {
            let new_value = record.value + amount;
            let result = sqlx::query(
                format!("UPDATE wallet SET {} = ? WHERE id = ? AND {} = ?", database_column, database_column).as_str())
                .bind(new_value)
                .bind(record.id)
                .bind(record.value)
                .execute(&ctx.data().database).await;

            if result.is_err() || result.unwrap().rows_affected() != 1 {
                return crate::commands::character_commands::send_stale_data_error(ctx).await
            }

            update_wallet_post(ctx, record.id).await;
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
            send_error(ctx, format!("Unable to find a wallet named {}.\n**Internal cache must be out of date. Please let me know if this ever happens.**", wallet.name).as_str()).await
        }
    }
}
