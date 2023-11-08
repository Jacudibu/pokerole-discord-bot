use crate::commands::{send_ephemeral_reply, Context, Error};
use chrono::{Datelike, Duration, FixedOffset, NaiveDate, TimeZone, Timelike, Utc};
use poise::ReplyHandle;
use serenity::utils::MessageBuilder;

/// Create a timestamp which automagically displays local time for everyone.
#[poise::command(slash_command)]
pub async fn timestamp(
    ctx: Context<'_>,
    #[description = "Which minute? Default now."] minute: Option<u8>,
    #[description = "Which hour? Default now."] hour: Option<u8>,
    #[description = "Which day? Default today."] day: Option<u8>,
    #[description = "Which month? Default this month."] month: Option<u8>,
    #[description = "Which year? Default this year."] year: Option<u16>,
) -> Result<(), Error> {
    let user_id = ctx.author().id.0 as i64;
    let user = sqlx::query!(
        "SELECT setting_time_offset_hours, setting_time_offset_minutes FROM user WHERE id = ?",
        user_id
    )
    .fetch_one(&ctx.data().database)
    .await;

    match user {
        Ok(user) => {
            if user.setting_time_offset_hours.is_none()
                || user.setting_time_offset_minutes.is_none()
            {
                send_settings_hint(&ctx).await?;
            }
            print_timestamp(
                &ctx,
                minute,
                hour,
                day,
                month,
                year,
                user.setting_time_offset_hours.unwrap_or_default(),
                user.setting_time_offset_minutes.unwrap_or_default(),
            )
            .await
        }
        Err(_) => {
            send_settings_hint(&ctx).await?;
            print_timestamp(&ctx, minute, hour, day, month, year, 0, 0).await
        }
    }
}

async fn send_settings_hint<'a>(ctx: &Context<'a>) -> Result<ReplyHandle<'a>, serenity::Error> {
    let current_datetime = Utc::now().naive_utc();
    send_ephemeral_reply(
        ctx,
        format!(
            "Looks like you don't have your timezone set up!
Right now, the time you need to enter is in UTC, which means **right now it is {}**.
In order to change this, use `/setting_time_offset` and select your local time from the list there.",
            current_datetime.format("%Y-%m-%d %H:%M")
        )
        .as_str(),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn print_timestamp<'a>(
    ctx: &Context<'a>,
    minute: Option<u8>,
    hour: Option<u8>,
    day: Option<u8>,
    month: Option<u8>,
    year: Option<u16>,
    hour_offset: i64,
    minute_offset: i64,
) -> Result<(), Error> {
    let second_offset = ((hour_offset * 60 + minute_offset) * 60) as i32;
    let offset = FixedOffset::east_opt(second_offset).expect("Should never be out of bounds");

    let local_datetime = offset.from_utc_datetime(&Utc::now().naive_utc());
    let minute = minute.unwrap_or(local_datetime.minute() as u8);
    let hour = hour.unwrap_or(local_datetime.hour() as u8);
    let day = day.unwrap_or(local_datetime.day() as u8);
    let month = month.unwrap_or(local_datetime.month() as u8);
    let year = year.unwrap_or(local_datetime.year() as u16);
    let timestamp = NaiveDate::from_ymd_opt(year.into(), month.into(), day.into())
        .expect("ymd should be valid!")
        .and_hms_opt(hour.into(), minute.into(), 0)
        .expect("hms should be valid!");

    let timestamp_utc = Utc.from_local_datetime(&timestamp).unwrap().naive_utc();
    let unix_timestamp = timestamp_utc.timestamp()
        - Duration::hours(hour_offset).num_seconds()
        - Duration::minutes(minute_offset).num_seconds();

    let result = std::format!("<t:{0}:f> (<t:{0}:R>)", unix_timestamp.to_string());
    let mut builder = MessageBuilder::default();
    builder.push_line(&result);
    builder.push_mono_line(&result);

    ctx.say(builder.to_string()).await?;
    Ok(())
}
