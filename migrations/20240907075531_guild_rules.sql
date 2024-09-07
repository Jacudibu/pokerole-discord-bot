CREATE TABLE guild_rules
(
    guild_id INTEGER NOT NULL,
    name     TEXT    NOT NULL COLLATE NOCASE,
    text     TEXT    NOT NULL,
    flavor   TEXT,
    example  TEXT,

    PRIMARY KEY (guild_id, name)
)
