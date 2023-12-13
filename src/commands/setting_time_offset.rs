use crate::commands::{Context, Error};
use chrono::{FixedOffset, Utc};
use serenity::builder::CreateSelectMenuOption;

fn build_select_menu_option(
    option: &mut CreateSelectMenuOption,
    hours: i32,
    minutes: i32,
) -> &mut CreateSelectMenuOption {
    let now = Utc::now();
    let offset = FixedOffset::east_opt(hours * 60 * 60 + minutes * 60)
        .expect("Should never be out of bounds");

    let result = now + offset;
    option.label(result.format("%v  |  %H:%M or %I:%M%P"));
    option.value(format!("{}_{}", hours, minutes));
    option
}

/// Open a dialogue to select your local timezone.
#[poise::command(slash_command)]
pub async fn setting_time_offset(ctx: Context<'_>) -> Result<(), Error> {
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

    ctx.send(|create_reply| {
        create_reply.ephemeral(true);
        create_reply.content(content).components(|components| {
            components.create_action_row(|row| {
                row.create_select_menu(|menu| {
                    menu.custom_id("timestamp-offset_UTC-X")
                        .placeholder("Select your local time here (UTC-X)")
                        .options(|f| {
                            f.create_option(|o| build_select_menu_option(o, -12, 0));
                            f.create_option(|o| build_select_menu_option(o, -11, 0));
                            f.create_option(|o| build_select_menu_option(o, -10, 0));
                            f.create_option(|o| build_select_menu_option(o, -9, -30));
                            f.create_option(|o| build_select_menu_option(o, -9, 0));
                            f.create_option(|o| build_select_menu_option(o, -8, 0));
                            f.create_option(|o| build_select_menu_option(o, -7, 0));
                            f.create_option(|o| build_select_menu_option(o, -6, 0));
                            f.create_option(|o| build_select_menu_option(o, -5, 0));
                            f.create_option(|o| build_select_menu_option(o, -4, 0));
                            f.create_option(|o| build_select_menu_option(o, -3, -30));
                            f.create_option(|o| build_select_menu_option(o, -3, 0));
                            f.create_option(|o| build_select_menu_option(o, -2, 0));
                            f.create_option(|o| build_select_menu_option(o, -1, 0));
                            f.create_option(|o| build_select_menu_option(o, 0, 0));
                            f
                        })
                })
            });
            components.create_action_row(|row| {
                row.create_select_menu(|menu| {
                    menu.custom_id("timestamp-offset_UTC+X")
                        .placeholder("Select your local time here (UTC+X)")
                        .options(|f| {
                            f.create_option(|o| build_select_menu_option(o, 0, 0));
                            f.create_option(|o| build_select_menu_option(o, 1, 0));
                            f.create_option(|o| build_select_menu_option(o, 2, 0));
                            f.create_option(|o| build_select_menu_option(o, 3, 0));
                            f.create_option(|o| build_select_menu_option(o, 3, 30));
                            f.create_option(|o| build_select_menu_option(o, 4, 0));
                            f.create_option(|o| build_select_menu_option(o, 4, 30));
                            f.create_option(|o| build_select_menu_option(o, 5, 0));
                            f.create_option(|o| build_select_menu_option(o, 5, 30));
                            f.create_option(|o| build_select_menu_option(o, 5, 45));
                            f.create_option(|o| build_select_menu_option(o, 6, 0));
                            f.create_option(|o| build_select_menu_option(o, 6, 30));
                            f.create_option(|o| build_select_menu_option(o, 7, 0));
                            f.create_option(|o| build_select_menu_option(o, 8, 0));
                            f.create_option(|o| build_select_menu_option(o, 8, 45));
                            f.create_option(|o| build_select_menu_option(o, 9, 0));
                            f.create_option(|o| build_select_menu_option(o, 9, 30));
                            f.create_option(|o| build_select_menu_option(o, 10, 0));
                            f.create_option(|o| build_select_menu_option(o, 10, 30));
                            f.create_option(|o| build_select_menu_option(o, 11, 0));
                            f.create_option(|o| build_select_menu_option(o, 12, 0));
                            f.create_option(|o| build_select_menu_option(o, 12, 45));
                            f.create_option(|o| build_select_menu_option(o, 13, 0));
                            f.create_option(|o| build_select_menu_option(o, 14, 0));
                            f
                        })
                })
            })
        })
    })
    .await?;

    Ok(())
}
