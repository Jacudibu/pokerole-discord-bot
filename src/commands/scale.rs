use std::default::Default;
use serenity::utils::MessageBuilder;
use crate::commands::{Context, Error};
use crate::commands::autocompletion::autocomplete_pokemon;

/// Scale a pokemon's size and weight!
#[poise::command(slash_command)]
pub async fn scale(
    ctx: Context<'_>,
    #[description = "Which pokemon?"]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
    #[description = "To which percentage? (Whole number)"]
    #[min = 1_u8]
    percent: u8,
) -> Result<(), Error> {
    if let Some(pokemon) = ctx.data().game.pokemon.get(&name.to_lowercase()) {
        let mut builder = MessageBuilder::new();
        builder.push_bold_line(std::format!("{} scaled to {}%", &pokemon.name, percent));
        builder.push_codeblock(std::format!("{}   |   {}",
                pokemon.height.scale(percent),
                pokemon.weight.scale(percent)
            ), None);
        ctx.say(builder.to_string()).await?;
    } else {
        ctx.send(|b| {
            b.content(std::format!("Unable to find a pokemon named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name));
            b.ephemeral(true)
        }).await?;
    }

    Ok(())
}
