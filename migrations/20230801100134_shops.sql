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

CREATE TABLE shop_owner(
    shop_id INTEGER NOT NULL,
    character_id INTEGER NOT NULL,
    PRIMARY KEY (shop_id, character_id),
    FOREIGN KEY (shop_id) REFERENCES shop(id),
    FOREIGN KEY (character_id) REFERENCES character(id)
);
