#[cfg(test)]
pub mod create_mock {
    use crate::data::Data;
    use crate::game_data;
    use chrono::Utc;
    use sqlx::{Pool, Sqlite};
    use std::sync::Arc;

    pub async fn data(db: Pool<Sqlite>) -> Data {
        // TODO: Only initialize game data arc once every cargo test run
        let game_data = game_data::parser::initialize_data().await;
        Data::new(db, Arc::new(game_data)).await
    }

    pub async fn user(db: &Pool<Sqlite>, user_id: i64) {
        let _ = sqlx::query!("INSERT INTO user (id) VALUES (?)", user_id)
            .execute(db)
            .await;
    }

    pub async fn guild(db: &Pool<Sqlite>, guild_id: i64) {
        let _ = sqlx::query!(
            "INSERT INTO guild (id, money, action_log_channel_id) VALUES (?, ?, ?)",
            guild_id,
            0,
            0
        )
        .execute(db)
        .await;
    }

    pub async fn quest(
        db: &Pool<Sqlite>,
        channel_id: i64,
        guild_id: i64,
        creator_id: i64,
        bot_message_id: i64,
    ) {
        let timestamp = Utc::now().timestamp();
        let _ = sqlx::query!("INSERT INTO quest (guild_id, channel_id, creator_id, bot_message_id, creation_timestamp) VALUES (?, ?, ?, ?, ?)",
            guild_id,
            channel_id,
            creator_id,
            bot_message_id,
            timestamp
        )
        .execute(db)
        .await;
    }

    pub(crate) async fn character(
        db: &Pool<Sqlite>,
        guild_id: i64,
        user_id: i64,
        character_id: i64,
        name: String,
    ) {
        let timestamp = Utc::now().timestamp();
        let _ = sqlx::query!("INSERT INTO character (id, user_id, guild_id, name, creation_date) VALUES (?, ?, ?, ?, ?)",
            character_id,
            user_id,
            guild_id,
            name,
            timestamp
        )
        .execute(db)
        .await;
    }
}
