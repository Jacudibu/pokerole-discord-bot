mod cache;
mod commands;
mod csv_utils;
mod data;
mod database_helpers;
mod emoji;
mod enums;
mod events;
mod game_data;
mod helpers;
mod logger;
mod parse_error;

use crate::data::Data;
use poise::builtins::on_error;
use poise::{serenity_prelude as serenity, FrameworkError};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;
use std::sync::Arc;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    logger::init_logging();

    let data = Data::new(
        initialize_database().await,
        Arc::new(game_data::parser::initialize_data().await),
    )
    .await;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::get_all_commands(),
            event_handler: |serenity_ctx, event, ctx, _| {
                Box::pin(events::handle_events(serenity_ctx, event, ctx))
            },
            on_error: |error| Box::pin(handle_error(error)),
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

async fn handle_error(error: FrameworkError<'_, Data, Error>) {
    match error {
        FrameworkError::Command { ctx, error } => {
            log::error!(
                "An error occurred in command /{}: {}",
                &ctx.command().name,
                error
            );
            if let Err(e) = ctx
                .send(|builder| {
                    builder
                        .ephemeral(true)
                        .reply(true)
                        .content(error.to_string())
                })
                .await
            {
                log::error!("Fatal error while sending error message: {}", e);
            }
        }
        _ => {
            if let Err(e) = on_error(error).await {
                log::error!("Fatal error while sending error message: {}", e);
            }
        }
    }
}

async fn initialize_database() -> Pool<Sqlite> {
    let url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::from_str(url.as_str())
                .expect("Unable to parse DATABASE_URL"),
        )
        .await
        .expect("Couldn't connect to database");

    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Couldn't run database migrations");
    database
}
