use crate::commands::Error;
use crate::shared::PoiseContext;
use chrono::{FixedOffset, Utc};
use poise::CreateReply;
use serenity::all::{CreateActionRow, CreateSelectMenu, CreateSelectMenuKind};
use serenity::builder::CreateSelectMenuOption;

fn build_select_menu_option(hours: i32, minutes: i32) -> CreateSelectMenuOption {
    let now = Utc::now();
    let offset = FixedOffset::east_opt(hours * 60 * 60 + minutes * 60)
        .expect("Should never be out of bounds");

    let result = now + offset;

    CreateSelectMenuOption::new(
        result.format("%v  |  %H:%M or %I:%M%P").to_string(),
        format!("{}_{}", hours, minutes),
    )
}

/// Open a dialogue to select your local timezone.
#[poise::command(slash_command)]
pub async fn setting_time_offset(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let user = sqlx::query!(
        "SELECT setting_time_offset_hours, setting_time_offset_minutes FROM user WHERE id = ?",
        user_id
    )
    .fetch_one(&ctx.data().database)
    .await;

    let mut content = "Select your local time from one of the two lists below.".to_string();
    if let Ok(user) = user {
        if let Some(hours) = user.setting_time_offset_hours {
            if let Some(minutes) = user.setting_time_offset_minutes {
                let now = Utc::now();
                let offset = FixedOffset::east_opt(hours as i32 * 60 * 60 + minutes as i32 * 60)
                    .expect("Should never be out of bounds");
                let result = now + offset;
                content = format!("According to your current setting, your local time should be {} right now. If this is not correct, you can select your local time from one of the two lists below.", result.format("%m-%d %H:%M"));
            }
        }
    }

    let utc_minus_x = CreateActionRow::SelectMenu(
        CreateSelectMenu::new(
            "timestamp-offset_UTC-X",
            CreateSelectMenuKind::String {
                options: vec![
                    build_select_menu_option(-12, 0),
                    build_select_menu_option(-11, 0),
                    build_select_menu_option(-10, 0),
                    build_select_menu_option(-9, -30),
                    build_select_menu_option(-9, 0),
                    build_select_menu_option(-8, 0),
                    build_select_menu_option(-7, 0),
                    build_select_menu_option(-6, 0),
                    build_select_menu_option(-5, 0),
                    build_select_menu_option(-4, 0),
                    build_select_menu_option(-3, -30),
                    build_select_menu_option(-3, 0),
                    build_select_menu_option(-2, 0),
                    build_select_menu_option(-1, 0),
                    build_select_menu_option(0, 0),
                ],
            },
        )
        .placeholder("Select your local time here (UTC-X)"),
    );

    let utc_plus_x = CreateActionRow::SelectMenu(
        CreateSelectMenu::new(
            "timestamp-offset_UTC+X",
            CreateSelectMenuKind::String {
                options: vec![
                    build_select_menu_option(0, 0),
                    build_select_menu_option(1, 0),
                    build_select_menu_option(2, 0),
                    build_select_menu_option(3, 0),
                    build_select_menu_option(3, 30),
                    build_select_menu_option(4, 0),
                    build_select_menu_option(4, 30),
                    build_select_menu_option(5, 0),
                    build_select_menu_option(5, 30),
                    build_select_menu_option(5, 45),
                    build_select_menu_option(6, 0),
                    build_select_menu_option(6, 30),
                    build_select_menu_option(7, 0),
                    build_select_menu_option(8, 0),
                    build_select_menu_option(8, 45),
                    build_select_menu_option(9, 0),
                    build_select_menu_option(9, 30),
                    build_select_menu_option(10, 0),
                    build_select_menu_option(10, 30),
                    build_select_menu_option(11, 0),
                    build_select_menu_option(12, 0),
                    build_select_menu_option(12, 45),
                    build_select_menu_option(13, 0),
                    build_select_menu_option(14, 0),
                ],
            },
        )
        .placeholder("Select your local time here (UTC+X)"),
    );

    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .content(content)
            .components(vec![utc_minus_x, utc_plus_x]),
    )
    .await?;

    Ok(())
}
