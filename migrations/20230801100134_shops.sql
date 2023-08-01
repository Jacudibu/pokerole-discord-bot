CREATE TABLE shop(
    name TEXT NOT NULL,
    guild_id INTEGER NOT NULL,
    bot_message_channel_id INTEGER NOT NULL,
    bot_message_id INTEGER NOT NULL,
    creation_timestamp INTEGER NOT NULL,
    money INTEGER DEFAULT 0,
    PRIMARY KEY (name, guild_id),
    FOREIGN KEY (guild_id) REFERENCES guild(id)
);
