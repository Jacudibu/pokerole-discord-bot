use crate::data::Data;
use crate::events::send_error_to_log_channel;
use chrono::{Duration, Utc};
use serenity::all::{CreateAttachment, CreateMessage};
use serenity::model::id::ChannelId;
use serenity::prelude::Context;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub async fn start_backup_thread(ctx: &Context, data: &Data) {
    if let Ok(backup_channel_id) = std::env::var("DB_BACKUP_CHANNEL_ID") {
        if let Ok(backup_channel_id) = backup_channel_id.parse() {
            let ctx = Arc::new(ctx.clone());
            if !data.is_backup_thread_running.load(Ordering::Relaxed) {
                let ctx_in_thread = Arc::clone(&ctx);
                tokio::spawn(async move {
                    loop {
                        upload_backup(Arc::clone(&ctx_in_thread), backup_channel_id).await;
                        tokio::time::sleep(calculate_duration_until_next_run()).await;
                    }
                });

                data.is_backup_thread_running.swap(true, Ordering::Relaxed);
            }
        } else {
            log::error!("Unable to parse DB_BACKUP_CHANNEL_ID into u64.")
        }
    } else {
        log::info!("Skipping database backups: DB_BACKUP_CHANNEL_ID is not defined.");
    }
}

fn calculate_duration_until_next_run() -> std::time::Duration {
    let now = Utc::now();
    let next_run = (now + Duration::days(1))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("Should never return None when passing 0's.");
    let seconds_until_midnight = next_run
        .signed_duration_since(now.naive_utc())
        .num_seconds()
        .unsigned_abs();

    let seconds_within_12_hours = Duration::hours(12).num_seconds().unsigned_abs();
    if seconds_until_midnight > seconds_within_12_hours {
        std::time::Duration::from_secs(seconds_until_midnight - seconds_within_12_hours)
    } else {
        std::time::Duration::from_secs(seconds_until_midnight)
    }
}

fn get_database_file_path() -> String {
    std::env::var("DATABASE_URL")
        .expect("missing DATABASE_URL")
        .replace("sqlite:///", "/")
        .replace("sqlite:", "")
}

async fn upload_backup(ctx: Arc<Context>, channel_id: u64) {
    let backup_channel_id = std::env::var("DB_BACKUP_CHANNEL_ID");
    if backup_channel_id.is_err() {
        return;
    }

    let database_path = get_database_file_path();
    let channel = ChannelId::from(channel_id);
    match tokio::fs::File::open(database_path).await {
        Ok(file) => {
            let filename = format!(
                "pokerolebot-backup-{}.sqlite",
                Utc::now().format("%Y-%m-%d-%H-%M")
            );
            match CreateAttachment::file(&file, filename).await {
                Ok(create_attachment) => {
                    let files = vec![create_attachment];

                    let result = channel.send_files(&ctx, files, CreateMessage::new()).await;
                    if let Err(e) = result {
                        let _ = channel
                            .send_message(
                                &ctx,
                                CreateMessage::new()
                                    .content(&format!("Failed to upload database backup: {}", e)),
                            )
                            .await;
                    }
                }
                Err(e) => {
                    send_error_to_log_channel(
                        &ctx,
                        &format!("Failed to create attachment for backup. Something went horribly wrong, I guess. File Path: {}, Internal Error: {:?}", get_database_file_path(), e),
                    )
                        .await;
                }
            }
        }
        Err(e) => {
            send_error_to_log_channel(
                &ctx,
                &format!(
                    "Failed to upload database backup, invalid file path: {}, internal error: {}",
                    get_database_file_path(),
                    e
                ),
            )
            .await;
        }
    }
}
