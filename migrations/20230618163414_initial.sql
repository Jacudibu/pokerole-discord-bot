CREATE TABLE characters(
    id INTEGER NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    guild_id INTEGER NOT NULL,
    name TEXT NOT NULL COLLATE NOCASE,
    stat_message_id INTEGER NOT NULL,
    stat_channel_id INTEGER NOT NULL,
    experience INTEGER NOT NULL,
    money INTEGER NOT NULL,

    UNIQUE(user_id, guild_id, name)
)
