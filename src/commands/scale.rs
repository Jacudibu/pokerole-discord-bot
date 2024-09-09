use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{pokemon_from_autocomplete_string, Context, Error};
use serenity::utils::MessageBuilder;
use std::default::Default;

/// Scale a pokemon's size and weight!
#[poise::command(slash_command)]
pub async fn scale(
    ctx: Context<'_>,
    #[description = "Which pokemon?"]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
    #[description = "To which percentage? (Whole number)"]
    #[min = 67_u8]
    #[max = 133_u8]
    percent: u8,
) -> Result<(), Error> {
    let pokemon = pokemon_from_autocomplete_string(&ctx, &name).await?;
    let mut builder = MessageBuilder::new();
    builder.push_bold_line(std::format!("{} scaled to {}%", &pokemon.name, percent));
    builder.push_codeblock(
        std::format!(
            "{}   |   {}",
            pokemon.height.scale(percent),
            pokemon.weight.scale(percent)
        ),
        None,
    );
    ctx.say(builder.to_string()).await?;

    Ok(())
}
