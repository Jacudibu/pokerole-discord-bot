use crate::Error;
use crate::commands::autocompletion::{autocomplete_rule, autocomplete_server_name};
use crate::commands::{get_servers_this_user_is_active_in, send_ephemeral_reply};
use crate::shared::PoiseContext;
use crate::shared::action_log::{ActionType, LogActionArguments, log_action};
use crate::shared::errors::{CommandInvocationError, DatabaseError, ValidationError};

/// Edit this server's rules.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("create_or_update", "delete", "clone"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn edit_rules(_: PoiseContext<'_>) -> Result<(), Error> {
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
    ctx: PoiseContext<'_>,
    #[description = "How should we call it?"] name: String,
    #[description = "What does the rule say?"] text: String,
    #[description = "A little flavor text for the rule?"] flavor: Option<String>,
    #[description = "Got an example?"] example: Option<String>,
) -> Result<(), Error> {
    validate("name", 100, &name)?;
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
            log_action(&ActionType::RuleUpdate,             LogActionArguments::triggered_by_user(&ctx),
                        format!("Created (or updated) a rule named {name}")).await?;
            Ok(())
        }
        Err(e) => Err(Box::new(DatabaseError::new(e.to_string()))),
    }
}

/// Delete an existing rule.
#[poise::command(prefix_command, slash_command)]
pub async fn delete(
    ctx: PoiseContext<'_>,
    #[description = "Which rule?"]
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
                LogActionArguments::triggered_by_user(&ctx),
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

/// Clone all rules from a different server. You need to own a character on the server to do this.
#[poise::command(prefix_command, slash_command)]
pub async fn clone(
    ctx: PoiseContext<'_>,
    #[description = "Which server?"]
    #[autocomplete = "autocomplete_server_name"]
    server_name: String,
) -> Result<(), Error> {
    let valid_servers = get_servers_this_user_is_active_in(&ctx).await?;
    let Some(position) = valid_servers.iter().position(|x| {
        x.name
            .as_ref()
            .is_some_and(|x| x.to_lowercase() == server_name.to_lowercase())
    }) else {
        return Err(Box::new(ValidationError::new(format!(
            "Unable to find a server named {server_name}. You need to own at least one character on it, and the server has to be set up with a server name."
        ))));
    };

    let guild_id = ctx.guild().expect("Command should be guild_only").id.get() as i64;
    if count_existing_guild_rules(&ctx, guild_id).await > 0 {
        return Err(Box::new(ValidationError::new(
            "You can only clone rules from another server when your own server has no rules set up!",
        )));
    }

    if let Some(server) = valid_servers.get(position) {
        send_ephemeral_reply(
            &ctx,
            format!("Selected server {:?} with id {}", server.name, server.id),
        )
        .await?;
        clone_all_rules(&ctx, server.id, guild_id).await;
        log_action(
            &ActionType::RuleClone,
            LogActionArguments::triggered_by_user(&ctx),
            format!("Cloned the rules from {:?}", server.name),
        )
        .await?;
        Ok(())
    } else {
        Err(Box::new(CommandInvocationError::new(
            "Something just went horribly wrong, hurray!",
        )))
    }
}

async fn count_existing_guild_rules(ctx: &PoiseContext<'_>, guild_id: i64) -> i64 {
    if let Ok(existing_rule_count) = sqlx::query!(
        "SELECT COUNT(*) as count FROM guild_rules WHERE guild_id = ?",
        guild_id
    )
    .fetch_one(&ctx.data().database)
    .await
    {
        existing_rule_count.count
    } else {
        0
    }
}

async fn clone_all_rules(ctx: &PoiseContext<'_>, from: i64, to: i64) {
    for record in sqlx::query!("SELECT * FROM guild_rules WHERE guild_id = ?", from)
        .fetch_all(&ctx.data().database)
        .await
        .unwrap()
    {
        let _ = sqlx::query!(
        "INSERT INTO guild_rules (guild_id, name, text, flavor, example) VALUES (?, ?, ?, ?, ?)",
        to,
        record.name,
        record.text,
        record.flavor,
        record.example
    )
        .execute(&ctx.data().database)
        .await;
    }
}

pub struct Rule {
    pub name: String,
    pub flavor: Option<String>,
    pub text: String,
    pub example: Option<String>,
}

impl Rule {
    pub fn build_string(&self) -> impl Into<String> + Sized {
        let mut builder = serenity::utils::MessageBuilder::default();
        builder.push(std::format!("**{}**\n", &self.name));
        if let Some(flavor) = &self.flavor {
            builder.push_italic_line(flavor);
            builder.push('\n');
        }

        builder.push(&self.text);

        if let Some(example) = &self.example {
            builder.push('\n');
            builder.quote_rest();
            builder.push(std::format!("**Example**: {}", example));
        }

        builder.build()
    }
}
