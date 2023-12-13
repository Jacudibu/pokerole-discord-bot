use crate::cache::{CharacterCacheItem, WalletCacheItem};
use crate::commands::autocompletion::{autocomplete_character_name, autocomplete_wallet_name};
use crate::commands::characters::{log_action, validate_user_input, ActionType};
use crate::commands::{
    ensure_guild_exists, find_character, find_wallet, send_ephemeral_reply, send_error, Context,
    Error,
};
use crate::data::Data;

/// Adds an existing character as owner for an existing wallet.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn add_wallet_owner(
    ctx: Context<'_>,
    #[description = "What's the wallet's name?"]
    #[autocomplete = "autocomplete_wallet_name"]
    wallet_name: String,
    #[autocomplete = "autocomplete_character_name"]
    #[description = "Which character should be added as owner?"]
    character_name: String,
) -> Result<(), Error> {
    if let Err(e) = validate_user_input(wallet_name.as_str()) {
        return send_error(&ctx, e).await;
    }

    let guild_id = ctx.guild_id().expect("Command is guild_only").get();
    ensure_guild_exists(&ctx, guild_id as i64).await;

    add_wallet_owner_impl(ctx.data(), &wallet_name, &character_name, guild_id).await?;

    send_ephemeral_reply(
        &ctx,
        format!(
            "Successfully added {} as owner for {}",
            character_name, wallet_name
        )
        .as_str(),
    )
    .await?;
    log_action(
        &ActionType::WalletChange,
        &ctx,
        &format!("Added {} as owner for {}", character_name, wallet_name,),
    )
    .await?;
    Ok(())
}

async fn add_wallet_owner_impl(
    data: &Data,
    wallet_name: &str,
    character_name: &str,
    guild_id: u64,
) -> Result<(), Error> {
    let character = find_character(data, guild_id, character_name).await?;
    let wallet = find_wallet(data, guild_id, wallet_name).await?;
    add_wallet_owner_to_db(wallet, character, data).await?;
    Ok(())
}

async fn add_wallet_owner_to_db(
    wallet: WalletCacheItem,
    character: CharacterCacheItem,
    data: &Data,
) -> Result<(), String> {
    let result = sqlx::query!(
        "INSERT INTO wallet_owner (wallet_id, character_id) VALUES (?, ?)",
        wallet.id,
        character.id,
    )
    .execute(&data.database)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 1 {
                Ok(())
            } else {
                Err("Something just went horribly wrong. Oh no!".to_string())
            }
        }
        Err(e) => Err(format!("**Something went wrong!**\n{}", e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::wallets::add_wallet_owner::add_wallet_owner_impl;
    use crate::{database_helpers, Error};
    use sqlx::{Pool, Sqlite};

    #[sqlx::test]
    async fn adding_wallet_owner_should_work(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let wallet_name = String::from("Test Wallet");
        let character_name = String::from("Test Character");
        let guild_id = 100;
        let user_id = 200;
        let wallet_id = 300;
        let character_id = 400;

        database_helpers::create_mock::guild(&data.database, guild_id).await;
        database_helpers::create_mock::user(&data.database, user_id).await;
        database_helpers::create_mock::wallet(&data.database, guild_id, wallet_id, &wallet_name)
            .await;
        database_helpers::create_mock::character(
            &data,
            guild_id,
            user_id,
            character_id,
            &character_name,
        )
        .await;

        add_wallet_owner_impl(&data, &wallet_name, &character_name, guild_id as u64).await?;

        let wallet_owners = sqlx::query!("SELECT wallet_id, character_id FROM wallet_owner")
            .fetch_all(&data.database)
            .await?;

        let wallet_owner = wallet_owners.first().unwrap();
        assert_eq!(character_id, wallet_owner.character_id);
        assert_eq!(wallet_id, wallet_owner.wallet_id);

        Ok(())
    }

    #[sqlx::test]
    async fn add_wallet_owner_called_twice_should_fail(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let wallet_name = String::from("Test Wallet");
        let character_name = String::from("Test Character");
        let guild_id = 100;
        let user_id = 200;
        let wallet_id = 300;
        let character_id = 400;

        database_helpers::create_mock::guild(&data.database, guild_id).await;
        database_helpers::create_mock::user(&data.database, user_id).await;
        database_helpers::create_mock::wallet(&data.database, guild_id, wallet_id, &wallet_name)
            .await;
        database_helpers::create_mock::character(
            &data,
            guild_id,
            user_id,
            character_id,
            &character_name,
        )
        .await;

        add_wallet_owner_impl(&data, &wallet_name, &character_name, guild_id as u64).await?;
        let result =
            add_wallet_owner_impl(&data, &wallet_name, &character_name, guild_id as u64).await;

        assert!(result.is_err());

        Ok(())
    }
}
