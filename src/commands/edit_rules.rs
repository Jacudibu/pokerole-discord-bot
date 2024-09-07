use crate::commands::autocompletion::autocomplete_rule;
use crate::commands::characters::{log_action, ActionType};
use crate::commands::{send_ephemeral_reply, Context};
use crate::errors::{DatabaseError, ValidationError};
use crate::Error;

/// Edit this server's rules.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("create_or_update", "delete"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn edit_rules(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

fn validate(variable_name: &str, max_length: usize, content: &str) -> Result<(), Error> {
    if content.len() > max_length {
        return Err(Box::new(ValidationError::new(format!(
            "Input Validation Error for {variable_name}: Too long."
        ))));
    }

    Ok(())
}

/// Create a new rule or update an existing one with the same name. Use \n for linebreaks.
#[poise::command(prefix_command, slash_command)]
pub async fn create_or_update(
    ctx: Context<'_>,
    #[description = "How should we call it?"] name: String,
    #[description = "What does the rule say?"] text: String,
    #[description = "A little flavor text for the rule?"] flavor: Option<String>,
    #[description = "Got an example?"] example: Option<String>,
) -> Result<(), Error> {
    validate("name", 30, &name)?;
    validate("text", 6000, &text)?;
    if let Some(flavor) = &flavor {
        validate("flavor", 6000, flavor)?;
    }
    if let Some(example) = &example {
        validate("example", 6000, example)?;
    }

    let guild_id = ctx.guild().expect("Command should be guild_only").id.get() as i64;
    let text = text.replace("\\n", "\n");
    let flavor = flavor.map(|x| x.replace("\\n", "\n"));
    let example = example.map(|x| x.replace("\\n", "\n"));

    match sqlx::query!(
        "INSERT INTO guild_rules (guild_id, name, text, flavor, example) VALUES (?, ?, ?, ?, ?)
ON CONFLICT (guild_id, name) DO UPDATE SET (text, flavor, example) = (excluded.text, excluded.flavor, excluded.example)",
        guild_id,
        name,
        text,
        flavor,
        example
    )
    .execute(&ctx.data().database)
    .await
    {
        Ok(_) => {
            send_ephemeral_reply(&ctx, "Rule was created (or updated)!").await?;
            log_action(&ActionType::RuleUpdate, &ctx, format!("Created (or updated) a rule named {name}")).await?;
            Ok(())
        }
        Err(e) => Err(Box::new(DatabaseError::new(e.to_string()))),
    }
}

/// Delete an existing rule.
#[poise::command(prefix_command, slash_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Which rule?"]
    #[rename = "name"]
    #[autocomplete = "autocomplete_rule"]
    name: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild().expect("Command should be guild_only").id.get() as i64;
    match sqlx::query!(
        "DELETE FROM guild_rules WHERE guild_id = ? AND name = ?",
        guild_id,
        name
    )
    .execute(&ctx.data().database)
    .await
    {
        Ok(_) => {
            send_ephemeral_reply(&ctx, "Rule was deleted!").await?;
            log_action(
                &ActionType::RuleDelete,
                &ctx,
                format!("Deleted a rule named {name}"),
            )
            .await?;
            Ok(())
        }
        Err(e) => Err(Box::new(DatabaseError::new(format!(
            "Was unable to delete a rule with name {name}: {e}"
        )))),
    }
}
