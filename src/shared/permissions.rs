use serenity::all::{Permissions, Role};

const DANGEROUS_PERMISSIONS: [Permissions; 19] = [
    Permissions::KICK_MEMBERS,
    Permissions::BAN_MEMBERS,
    Permissions::ADMINISTRATOR,
    Permissions::MANAGE_CHANNELS,
    Permissions::VIEW_AUDIT_LOG,
    Permissions::MANAGE_MESSAGES,
    Permissions::MUTE_MEMBERS,
    Permissions::DEAFEN_MEMBERS,
    Permissions::MOVE_MEMBERS,
    Permissions::MANAGE_NICKNAMES,
    Permissions::MANAGE_ROLES,
    Permissions::MANAGE_WEBHOOKS,
    Permissions::MANAGE_GUILD_EXPRESSIONS,
    Permissions::MANAGE_EVENTS,
    Permissions::MANAGE_THREADS,
    Permissions::MODERATE_MEMBERS,
    Permissions::VIEW_CREATOR_MONETIZATION_ANALYTICS,
    Permissions::CREATE_GUILD_EXPRESSIONS,
    Permissions::CREATE_EVENTS,
];

pub fn does_role_have_dangerous_permissions(role: &Role) -> bool {
    for permission in DANGEROUS_PERMISSIONS {
        if role.has_permission(permission) {
            return true;
        }
    }

    false
}
