use std::sync::atomic::Ordering;
use std::sync::Arc;

use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use log::info;
use serenity::all::CreateMessage;
use serenity::model::id::ChannelId;
use serenity::prelude::Context;
use sqlx::{Pool, Sqlite};

use crate::data::Data;
use crate::events::character_stat_edit::update_character_post;
use crate::events::send_error_to_log_channel;

pub async fn start_monthly_reset_thread(ctx: &Context, data: &Data) {
    let ctx = Arc::new(ctx.clone());
    if !data.is_monthly_reset_thread_running.load(Ordering::Relaxed) {
        let ctx_in_thread = Arc::clone(&ctx);
        let database = data.database.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(calculate_duration_until_next_run()).await;
                execute_monthly_reset(Arc::clone(&ctx_in_thread), database.clone()).await;
            }
        });

        data.is_monthly_reset_thread_running
            .swap(true, Ordering::Relaxed);
    }
}

fn calculate_duration_until_next_run() -> std::time::Duration {
    let now = Utc::now();

    let next_year = if now.month() == 12 {
        now.year() + 1
    } else {
        now.year()
    };
    let next_month = if now.month() == 12 {
        1
    } else {
        now.month() + 1
    };

    let seconds_until_next_run = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("Date for next month should always be valid")
        .and_hms_opt(0, 1, 0)
        .expect("Date for next month should always be valid")
        .signed_duration_since(now.naive_utc())
        .num_seconds()
        .unsigned_abs();

    info!("{}", seconds_until_next_run);

    std::time::Duration::from_secs(seconds_until_next_run)
}

async fn execute_monthly_reset(ctx: Arc<Context>, database: Pool<Sqlite>) {
    match sqlx::query!(
        "UPDATE character SET 
tera_used_normal = 0,
tera_used_fighting = 0,
tera_used_flying = 0,
tera_used_poison = 0,
tera_used_ground = 0,
tera_used_rock = 0,
tera_used_bug = 0,
tera_used_ghost = 0,
tera_used_steel = 0,
tera_used_fire = 0,
tera_used_water = 0,
tera_used_grass = 0,
tera_used_electric = 0,
tera_used_psychic = 0,
tera_used_ice = 0,
tera_used_dragon = 0,
tera_used_dark = 0,
tera_used_fairy = 0
"
    )
    .execute(&database)
    .await
    {
        Ok(_) => {
            notify_guilds(&ctx, &database).await;
            // Updating character posts is disabled until we figure out how to reopen forum threads without sending a message...
        }
        Err(error) => {
            send_error_to_log_channel(&ctx, error.to_string()).await;
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
                        CreateMessage::new().content("📅 [System] Performing monthly reset."),
                    )
                    .await;
            }
        }
        Err(error) => {
            send_error_to_log_channel(ctx, error.to_string()).await;
        }
    }
}