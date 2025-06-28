use crate::commands::autocompletion::autocomplete_wallet_name;
use crate::commands::wallet_commands::update_wallet_post;
use crate::commands::{Error, find_wallet, send_ephemeral_reply, send_error};
use crate::shared::PoiseContext;
use crate::shared::action_log::{ActionType, LogActionArguments, log_action};

/// Update wallet data. All arguments are optional.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn edit_wallet(
    ctx: PoiseContext<'_>,
    #[description = "Which wallet?"]
    #[autocomplete = "autocomplete_wallet_name"]
    wallet: String,
    #[description = "Change the name?"] name: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    let wallet = find_wallet(ctx.data(), guild_id, &wallet).await?;

    let record = sqlx::query!("SELECT name FROM wallet WHERE id = ?", wallet.id)
        .fetch_one(&ctx.data().database)
        .await?;

    let mut action_log = Vec::new();

    let name = if let Some(name) = name {
        action_log.push(format!("name to {}", name));
        name
    } else {
        record.name
    };

    if action_log.is_empty() {
        send_error(&ctx, "No changes requested, aborting.").await?;
        return Ok(());
    }

    sqlx::query!("UPDATE wallet SET name = ? WHERE id = ?", name, wallet.id,)
        .execute(&ctx.data().database)
        .await?;

    update_wallet_post(&ctx, wallet.id).await;

    let action_log = action_log.join(", ");
    let _ = log_action(
        &ActionType::WalletEdit,
        LogActionArguments::triggered_by_user(&ctx),
        &format!("Set {}'s {}.", wallet.name, action_log),
    )
    .await;
    let _ = send_ephemeral_reply(&ctx, &format!("Updated {}'s {}.", wallet.name, action_log)).await;
    Ok(())
}
