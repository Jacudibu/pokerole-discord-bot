use crate::commands::autocompletion::autocomplete_rule;
use crate::commands::{edit_rules, Error};
use crate::shared::errors::ValidationError;
use crate::shared::utility::message_splitting;
use crate::shared::PoiseContext;

/// Display rule
#[poise::command(slash_command, guild_only)]
pub async fn rule(
    ctx: PoiseContext<'_>,
    #[description = "Which rule?"]
    #[rename = "name"]
    #[autocomplete = "autocomplete_rule"]
    name: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get() as i64;

    match sqlx::query_as!(
        edit_rules::Rule,
        "SELECT name, text, flavor, example FROM guild_rules WHERE guild_id = ? AND name = ?",
        guild_id,
        name,
    )
    .fetch_one(&ctx.data().database)
    .await
    {
        Ok(rule) => {
            for split in message_splitting::split_long_messages(rule.build_string().into()) {
                if let Err(e) = ctx.say(split).await {
                    let _ = ctx
                        .reply(&format!("Encountered an unexpected error:\n```{}```", e))
                        .await;
                }
            }
            Ok(())
        }

        Err(_) => Err(Box::new(ValidationError::new(format!(
            "Unable to find a rule named **{}** on this server, sorry!",
            name
        )))),
    }
}
