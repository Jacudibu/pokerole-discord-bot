use crate::commands::autocompletion::autocomplete_rule;
use crate::commands::{Context, Error};
use poise::CreateReply;

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
        ctx.send(
            CreateReply::default()
                .content(std::format!(
                    "Unable to find a rule named **{}**, sorry!",
                    name
                ))
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}
