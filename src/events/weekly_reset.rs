use crate::data::Data;
use crate::events::send_error_to_log_channel;
use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use serenity::all::CreateMessage;
use serenity::model::id::ChannelId;
use serenity::prelude::Context;
use sqlx::{Pool, Sqlite};
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub async fn start_weekly_reset_thread(ctx: &Context, data: &Data) {
    let ctx = Arc::new(ctx.clone());
    if !data.is_weekly_reset_thread_running.load(Ordering::Relaxed) {
        let ctx_in_thread = Arc::clone(&ctx);
        let database = data.database.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(calculate_duration_until_next_run()).await;
                execute_weekly_reset(Arc::clone(&ctx_in_thread), database.clone()).await;
            }
        });

        data.is_weekly_reset_thread_running
            .swap(true, Ordering::Relaxed);
    }
}

fn calculate_duration_until_next_run() -> std::time::Duration {
    let now = Utc::now();
    let now_iso = now.iso_week();

    let last_monday = NaiveDate::from_isoywd_opt(now_iso.year(), now_iso.week(), Weekday::Mon)
        .expect("Week 1 Monday should always exist, even if it's within the last year.");

    let next_run = (last_monday + Duration::days(7))
        .and_hms_opt(0, 0, 0)
        .expect("Should never return None when passing 0's.");
    let seconds_until_next_run = next_run
        .signed_duration_since(now.naive_utc())
        .num_seconds()
        .unsigned_abs();

    std::time::Duration::from_secs(seconds_until_next_run)
}

async fn execute_weekly_reset(ctx: Arc<Context>, database: Pool<Sqlite>) {
    match sqlx::query!("UPDATE character SET weekly_spar_count = 0")
        .execute(&database)
        .await
    {
        Ok(_) => {
            notify_guilds(&ctx, &database).await;
            update_character_posts(&ctx, &database).await;
        }
        Err(error) => {
            send_error_to_log_channel(&ctx, error.to_string()).await;
        }
    }
}

async fn update_character_posts(ctx: &Arc<Context>, database: &Pool<Sqlite>) {
    match sqlx::query!("SELECT id FROM character")
        .fetch_all(database)
        .await
    {
        Ok(records) => {
            for x in records {
                todo!();
            }
        }
        Err(error) => {
            send_error_to_log_channel(ctx, error.to_string()).await;
        }
    }
}

async fn notify_guilds(ctx: &Arc<Context>, database: &Pool<Sqlite>) {
    match sqlx::query!("SELECT action_log_channel_id FROM guild")
        .fetch_all(database)
        .await
    {
        Ok(records) => {
            let channel_ids: Vec<i64> = records
                .iter()
                .filter_map(|x| x.action_log_channel_id)
                .collect();

            for action_log_channel_id in channel_ids {
                let channel = ChannelId::from(action_log_channel_id as u64);
                let _ = channel
                    .send_message(
                        &ctx,
                        CreateMessage::new().content("📅 [System] Performing weekly reset."),
                    )
                    .await;
            }
        }
        Err(error) => {
            send_error_to_log_channel(ctx, error.to_string()).await;
        }
    }
}
