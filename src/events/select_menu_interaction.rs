use crate::errors::ParseError;
use crate::events::FrameworkContext;
use crate::{events, Error};
use serenity::all::{ComponentInteraction, ComponentInteractionDataKind};
use serenity::client::Context;

pub async fn handle_select_menu_interaction(
    context: &Context,
    framework: FrameworkContext<'_>,
    interaction: &&ComponentInteraction,
) -> Result<(), Error> {
    if interaction.data.custom_id.is_empty() {
        return Ok(());
    }

    let (command, _) = events::parse_interaction_command(interaction.data.custom_id.as_str());
    if command == "timestamp-offset" {
        timestamp_offset(context, framework, interaction).await
    } else {
        Ok(())
    }
}

async fn timestamp_offset(
    context: &Context,
    framework: FrameworkContext<'_>,
    interaction: &&ComponentInteraction,
) -> Result<(), Error> {
    let selected_value = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values.first(),
        _ => None,
    };

    if selected_value.is_none() {
        return Err(Box::new(ParseError::new(&format!(
            "Unable to parse selected value. Weird stuff. Interaction data: {:?}",
            &interaction.data.kind
        ))));
    }

    let args: Vec<i32> = selected_value
        .unwrap()
        .split('_')
        .map(|x| x.parse().expect("Arguments should never be invalid."))
        .collect();
    let hours = args[0];
    let minutes = args[1];

    let user_id = interaction.user.id.get() as i64;
    let user = sqlx::query!(
        "SELECT setting_time_offset_hours, setting_time_offset_minutes FROM user WHERE id = ?",
        user_id
    )
    .fetch_one(&framework.user_data.database)
    .await;

    match user {
        Ok(_) => {
            let result = sqlx::query!(
                "UPDATE user SET setting_time_offset_hours = ?, setting_time_offset_minutes = ? WHERE id = ?",
                hours,
                minutes,
                user_id
                ).execute(&framework.user_data.database).await;
            if result.is_ok() && result.unwrap().rows_affected() == 1 {
                events::send_ephemeral_reply(
                    interaction,
                    context,
                    "Successfully set your local time!",
                )
                .await?;
                Ok(())
            } else {
                events::send_error(
                    interaction,
                    context,
                    "Unable to update your time offsets. Mh! Weird.",
                )
                .await?;
                Ok(())
            }
        }
        Err(_) => {
            let result = sqlx::query!(
                "INSERT INTO user (id, setting_time_offset_hours, setting_time_offset_minutes) VALUES (?, ?, ?) RETURNING id",
                user_id,
                hours,
                minutes,
                ).fetch_one(&framework.user_data.database).await;

            if result.is_ok() {
                events::send_ephemeral_reply(
                    interaction,
                    context,
                    "Successfully set your local time!",
                )
                .await?;
                Ok(())
            } else {
                events::send_error(
                    interaction,
                    context,
                    "Unable to create a user entry for you. Mh! Weird.",
                )
                .await?;
                Ok(())
            }
        }
    }
}
