use crate::commands::{send_ephemeral_reply, send_error, Context, Error};
use chrono::{FixedOffset, Timelike, Utc};
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
    option.label(result.format("%m-%d %H:%M"));
    option.value(format!("timestamp_{}_{}", hours, minutes));
    option
}

/// Set an offset for the timestamp command.
#[poise::command(slash_command)]
pub async fn setting_time_offset(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.0 as i64;
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
        create_reply.content(content).components(|components| {
            components.create_action_row(|row| {
                row.create_select_menu(|menu| {
                    menu.custom_id("timestamp-selection-behind")
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
                    menu.custom_id("timestamp-selection-ahead")
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

    // match user {
    //     Ok(_) => {
    //         let result = sqlx::query!(
    //             "UPDATE user SET setting_time_offset_hours = ?, setting_time_offset_minutes = ? WHERE id = ?",
    //             hours,
    //             minutes,
    //             user_id
    //         ).execute(&ctx.data().database).await;
    //
    //         if result.is_ok() && result.unwrap().rows_affected() == 1 {
    //             send_ephemeral_reply(&ctx, "Successfully set your local time!").await?;
    //             Ok(())
    //         } else {
    //             send_error(&ctx, "Unable to update your time offsets. Mh! Weird.").await?;
    //             Ok(())
    //         }
    //     }
    //     Err(_) => {
    //         let result = sqlx::query!(
    //             "INSERT INTO user (id, setting_time_offset_hours, setting_time_offset_minutes) VALUES (?, ?, ?) RETURNING id",
    //             user_id,
    //             hours,
    //             minutes,
    //         ).fetch_one(&ctx.data().database).await;
    //
    //         if result.is_ok() {
    //             send_ephemeral_reply(&ctx, "Successfully set your local time!").await?;
    //             Ok(())
    //         } else {
    //             send_error(&ctx, "Unable to create a user entry for you. Mh! Weird.").await?;
    //             Ok(())
    //         }
    //     }
    //}
}
