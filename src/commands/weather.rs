use crate::commands::autocompletion::autocomplete_weather;
use crate::commands::{Context, Error};
use poise::CreateReply;

/// Display the Weather
#[poise::command(slash_command)]
pub async fn weather(
    ctx: Context<'_>,
    #[description = "Which weather?"]
    #[rename = "name"]
    #[autocomplete = "autocomplete_weather"]
    name: String,
) -> Result<(), Error> {
    if let Some(weather) = ctx.data().game.weather.get(&name.to_lowercase()) {
        ctx.say(weather.build_string()).await?;
    } else {
        ctx.send(CreateReply::default()
            .content(std::format!("Unable to find a weather condition named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name))
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}
