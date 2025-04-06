use serenity::all::ChannelId;

pub const ADMIN_PING_STRING: &str = "<@878982444412448829>";
pub const ERROR_LOG_CHANNEL: ChannelId = ChannelId::new(1188864512439369779);
pub const DEFAULT_BACKPACK_SLOTS: i64 = 6;
pub const DISCORD_MESSAGE_LENGTH_LIMIT: usize = 2000;

pub mod discord_error_codes {
    pub const ARCHIVED_THREAD: isize = 50083;
}
