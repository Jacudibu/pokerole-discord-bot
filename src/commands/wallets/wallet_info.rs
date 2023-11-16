use crate::commands::autocompletion::autocomplete_wallet_name;
use crate::commands::wallets::build_wallet_string;
use crate::commands::{find_wallet, Context, Error};
use crate::errors::ParseError;

/// Have a look into a wallet.
#[poise::command(slash_command, guild_only)]
pub async fn wallet_info(
    ctx: Context<'_>,
    #[description = "What's the wallet's name?"]
    #[autocomplete = "autocomplete_wallet_name"]
    wallet: String,
) -> Result<(), Error> {
    // TODO: Button to undo the transaction which lasts for a minute or so.
    let guild_id = ctx.guild_id().expect("Command is guild_only").0;
    let wallet = find_wallet(ctx.data(), guild_id, &wallet).await?;

    let result = build_wallet_string(&ctx, wallet.id).await;
    if let Some(result) = result {
        ctx.reply(result.message).await?;
    } else {
        return Err(Box::new(ParseError::new(
            "Something just went horribly wrong. :D",
        )));
    }
    Ok(())
}
