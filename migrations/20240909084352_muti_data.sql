ALTER TABLE guild
    ADD COLUMN data_source_id INTEGER;

ALTER TABLE user
    ADD COLUMN last_data_source_id INTEGER NOT NULL DEFAULT 0;