DROP TABLE emoji_guild;

CREATE TABLE application_emoji
(
    species_api_id INTEGER NOT NULL,
    discord_string TEXT    NOT NULL,
    UNIQUE (species_api_id)
);
