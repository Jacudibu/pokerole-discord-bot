use crate::commands::create_role_reaction_post::CreateRoleError::RoleWasNone;
use crate::commands::{Error, send_ephemeral_reply};
use crate::shared::utility::button_building::create_button;
use crate::shared::{PoiseContext, permissions};
use serenity::all::{CreateActionRow, CreateMessage};
use serenity::builder::CreateButton;
use serenity::model::guild::Role;

/// Create a post that allows users to assign non-administrative roles to themselves.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
#[allow(clippy::too_many_arguments)]
pub async fn create_role_reaction_post(
    ctx: PoiseContext<'_>,
    #[description = "What should the message say?"] message_text: String,
    #[description = "The emoji for the first role"] emoji_1: Option<String>,
    #[description = "The role for the first role"] role_1: Role,
    emoji_2: Option<String>,
    role_2: Option<Role>,
    emoji_3: Option<String>,
    role_3: Option<Role>,
    emoji_4: Option<String>,
    role_4: Option<Role>,
    emoji_5: Option<String>,
    role_5: Option<Role>,
    emoji_6: Option<String>,
    role_6: Option<Role>,
    emoji_7: Option<String>,
    role_7: Option<Role>,
    emoji_8: Option<String>,
    role_8: Option<Role>,
) -> Result<(), Error> {
    let channel = ctx.channel_id();
    let mut issues = String::new();

    let buttons = vec![
        add_button(emoji_1, role_1.into()),
        add_button(emoji_2, role_2),
        add_button(emoji_3, role_3),
        add_button(emoji_4, role_4),
        add_button(emoji_5, role_5),
        add_button(emoji_6, role_6),
        add_button(emoji_7, role_7),
        add_button(emoji_8, role_8),
    ]
    .into_iter()
    .flat_map(|x| match x {
        Ok(x) => Some(x),
        Err(e) => match e {
            RoleWasNone => None,
            CreateRoleError::EvilPermission(role) => {
                issues.push_str(&format!(
                    "Role with dangerous permissions detected which should not be self-assignable by everyone: @{role}. I'll skip that one.\n"
                ));
                None
            }
        },
    })
    .collect::<Vec<CreateButton>>()
    .chunks(4)
    .map(|chunk| CreateActionRow::Buttons(chunk.to_vec()))
    .collect::<Vec<CreateActionRow>>();

    let _ = send_ephemeral_reply(&ctx, if issues.is_empty() { "ok" } else { &issues }).await;
    let _ = channel
        .send_message(
            ctx,
            CreateMessage::new()
                .content(format!("### {message_text}"))
                .components(buttons),
        )
        .await?;

    Ok(())
}

enum CreateRoleError {
    /// That's to be expected for optional roles.
    RoleWasNone,
    /// Evil role permissions.
    EvilPermission(String),
}

fn add_button(emoji: Option<String>, role: Option<Role>) -> Result<CreateButton, CreateRoleError> {
    let Some(role) = role else {
        return Err(RoleWasNone);
    };

    if permissions::does_role_have_dangerous_permissions(&role) {
        return Err(CreateRoleError::EvilPermission(role.name));
    }

    let emoji = emoji.unwrap_or_default();

    Ok(create_button(
        format!("{emoji} @{}", role.name).trim(),
        &format!("toggle-role_{}", role.id),
        false,
    ))
}
