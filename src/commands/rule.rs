use crate::commands::{Context, Error};
use crate::commands::autocompletion::autocomplete_rule;

/// Display rule
#[poise::command(slash_command)]
pub async fn rule(
    ctx: Context<'_>,
    #[description = "Which rule?"]
    #[rename = "name"]
    #[autocomplete = "autocomplete_rule"]
    name: String,
) -> Result<(), Error> {
    if let Some(rule) = ctx.data().game.rules.get(&name.to_lowercase()) {
        ctx.say(rule.build_string()).await?;
    } else {
        ctx.send(|b| {
            b.content(std::format!("Unable to find a rule named **{}**, sorry!", name));
            b.ephemeral(true)
        }).await?;
    }

    Ok(())
}
