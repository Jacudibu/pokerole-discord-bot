use crate::commands::character_commands::validate_user_input;
use crate::commands::{Error, send_ephemeral_reply};
use crate::shared::PoiseContext;
use crate::shared::action_log::{ActionType, LogActionArguments, log_action};
use crate::shared::errors::CommandInvocationError;
use serenity::all::{Mention, Role, RoleId};
use serenity::model::channel::Channel;

/// Register this server within the database, or change values that have been set up earlier.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn setup_guild(
    ctx: PoiseContext<'_>,
    name: Option<String>,
    action_log_channel: Option<Channel>,
    default_member_role: Option<Role>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get() as i64;

    let name = if let Some(name) = name {
        validate_user_input(&name)?;
        Some(name)
    } else {
        None
    };
    let action_log_channel_id = action_log_channel.map(|x| x.id().get() as i64);
    let default_member_role_id = default_member_role.map(|x| x.id.get() as i64);

    match sqlx::query!(
        "INSERT INTO guild (id, name, action_log_channel_id, default_member_role_id) VALUES (?, ?, ?, ?)
ON CONFLICT (id) DO UPDATE SET
    name = excluded.name,
    action_log_channel_id = excluded.action_log_channel_id,
    default_member_role_id = excluded.default_member_role_id
RETURNING *",
        guild_id,
        name,
        action_log_channel_id,
        default_member_role_id
    )
        .fetch_one(&ctx.data().database)
        .await {
        Ok(record) => {
            send_ephemeral_reply(&ctx, "Guild has been successfully set up!").await?;
            log_action(&ActionType::Initialization,
                       LogActionArguments::triggered_by_user(&ctx),
                       "The action log channel has been set to this lovely place here. I recommend muting this channel, lul.").await?;
            if let Some(name) = record.name {
                log_action(
                    &ActionType::Initialization,
                    LogActionArguments::triggered_by_user(&ctx),
                    format!("Guild Name has been set to **{name}**"),
                )
                    .await?;
            }
            if let Some(default_member_role_id) = record.default_member_role_id {
                let role = RoleId::new(default_member_role_id as u64);
                let mention = Mention::Role(role);
                log_action(
                    &ActionType::Initialization,
                    LogActionArguments::triggered_by_user(&ctx),
                    format!("Default Member Role has been set to **{mention}**"),
                )
                    .await?;
            }

            Ok(())
        }
        Err(e) => {
            Err(Box::new(CommandInvocationError::new(format!("Something went wrong! {}", e))))
        }
    }
}
