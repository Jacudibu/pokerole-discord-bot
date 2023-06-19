mod commands;
mod game_data;
mod logger;
mod csv_utils;
mod enums;
mod data;
mod parse_error;
mod events;
mod helpers;
mod cache;
mod emoji;

use std::str::FromStr;
use std::sync::Arc;
use poise::serenity_prelude as serenity;
use sqlx::{Pool, Sqlite};
use crate::data::Data;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    logger::init_logging();

    let data = Data::new(
        initialize_database().await,
        Arc::new(game_data::parser::initialize_data().await)
    ).await;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::get_all_commands(),
            event_handler: |serenity_ctx, event, ctx, _| Box::pin(events::handle_events(serenity_ctx, event, ctx)),
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        });

    framework.run().await.unwrap();
}

async fn initialize_database() -> Pool<Sqlite> {
    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::from_str(url.as_str()).expect("Unable to parse DATABASE_URL")
        )
        .await
        .expect("Couldn't connect to database");

    sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");
    database
}
