CREATE TABLE quest(
    guild_id INTEGER NOT NULL,
    channel_id INTEGER NOT NULL PRIMARY KEY,
    creator_id INTEGER NOT NULL,
    bot_message_id INTEGER NOT NULL,
    creation_timestamp INTEGER NOT NULL,
    completion_timestamp INTEGER,
    FOREIGN KEY (guild_id) REFERENCES guild(id),
    FOREIGN KEY (creator_id) REFERENCES user(id)
);

CREATE TABLE quest_signup(
    quest_id INTEGER NOT NULL,
    character_id INTEGER NOT NULL,
    creation_timestamp INTEGER NOT NULL,
    PRIMARY KEY (character_id, quest_id),
    FOREIGN KEY (character_id) REFERENCES character(id),
    FOREIGN KEY (quest_id) REFERENCES quest(channel_id)
);

CREATE TABLE quest_completion(
    quest_id INTEGER NOT NULL,
    character_id INTEGER NOT NULL,
    creation_timestamp INTEGER NOT NULL,
    PRIMARY KEY (character_id, quest_id),
    FOREIGN KEY (character_id) REFERENCES character(id),
    FOREIGN KEY (quest_id) REFERENCES quest(channel_id)
);
