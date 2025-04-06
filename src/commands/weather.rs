use crate::commands::autocompletion::autocomplete_weather;
use crate::commands::Error;
use crate::shared::PoiseContext;
use poise::CreateReply;

/// Display the Weather
#[poise::command(slash_command)]
pub async fn weather(
    ctx: PoiseContext<'_>,
    #[description = "Which weather?"]
    #[rename = "name"]
    #[autocomplete = "autocomplete_weather"]
    name: String,
) -> Result<(), Error> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;

    if let Some(weather) = game_data.weather.get(&name.to_lowercase()) {
        ctx.say(weather.build_string()).await?;
    } else {
        ctx.send(CreateReply::default()
            .content(std::format!("Unable to find a weather condition named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name))
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}
