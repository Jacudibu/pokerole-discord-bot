CREATE TABLE shop(
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    guild_id INTEGER NOT NULL,
    bot_message_channel_id INTEGER NOT NULL,
    bot_message_id INTEGER NOT NULL,
    creation_timestamp INTEGER NOT NULL,
    money INTEGER NOT NULL,
    FOREIGN KEY (guild_id) REFERENCES guild(id),
    UNIQUE(name, guild_id)
);
