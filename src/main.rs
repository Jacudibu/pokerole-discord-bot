mod commands;
mod data;
mod logger;
mod csv_utils;
mod enums;
mod game_data;
mod parse_error;
mod events;
mod helpers;

use poise::serenity_prelude as serenity;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    logger::init_logging();
    let data = data::parser::initialize_data().await;

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
