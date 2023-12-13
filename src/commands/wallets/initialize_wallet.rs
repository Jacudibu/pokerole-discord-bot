use crate::commands::characters::{log_action, validate_user_input, ActionType};
use crate::commands::wallets::update_wallet_post;
use crate::commands::{ensure_guild_exists, send_ephemeral_reply, send_error, Context, Error};
use crate::data::Data;
use crate::emoji;
use chrono::Utc;
use serenity::all::CreateMessage;

/// Create a new wallet within the database.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn initialize_wallet(
    ctx: Context<'_>,
    #[description = "What name should we use?"] name: String,
    #[min = 0_i64] money: Option<i64>,
) -> Result<(), Error> {
    if let Err(e) = validate_user_input(name.as_str()) {
        return send_error(&ctx, e).await;
    }

    let money = money.unwrap_or(0);
    let message = ctx
        .channel_id()
        .send_message(
            ctx,
            CreateMessage::new().content(
                "[Placeholder. This should get replaced or deleted within a couple seconds.]",
            ),
        )
        .await?;

    let guild_id = ctx.guild_id().expect("Command is guild_only").get() as i64;

    ensure_guild_exists(&ctx, guild_id).await;

    let message_id = message.id.get() as i64;
    let channel_id = message.channel_id.get() as i64;

    let result = create_wallet(
        name.clone(),
        guild_id,
        message_id,
        channel_id,
        money,
        ctx.data(),
    )
    .await;
    if let Ok(id) = result {
        send_ephemeral_reply(&ctx, "Wallet has been successfully created!").await?;
        update_wallet_post(&ctx, id).await;
        log_action(
            &ActionType::Initialization,
            &ctx,
            &format!(
                "Created a wallet for {} with {} {}.",
                name,
                money,
                emoji::POKE_COIN,
            ),
        )
        .await?;
        ctx.data()
            .cache
            .update_character_names(&ctx.data().database)
            .await;
        return Ok(());
    }

    send_error(
        &ctx,
        "Something went wrong! Does a wallet with this name already exist on this server?",
    )
    .await?;
    message.delete(ctx).await?;

    Ok(())
}

async fn create_wallet(
    name: String,
    guild_id: i64,
    message_id: i64,
    channel_id: i64,
    money: i64,
    data: &Data,
) -> Result<i64, String> {
    let timestamp = Utc::now().timestamp();

    let result = sqlx::query!(
        "INSERT INTO wallet (name, guild_id, bot_message_id, bot_message_channel_id, creation_timestamp, money) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
        name,
        guild_id,
        message_id,
        channel_id,
        timestamp,
        money
    ).fetch_one(&data.database)
        .await;

    match result {
        Ok(result) => Ok(result.id),
        Err(e) => Err(format!("**Something went wrong!**\n{}", e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::wallets::initialize_wallet::create_wallet;
    use crate::{database_helpers, Error};
    use chrono::Utc;
    use more_asserts::{assert_ge, assert_le};
    use sqlx::{Pool, Sqlite};

    #[sqlx::test]
    async fn create_wallet_should_work(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let name = String::from("Test Wallet");
        let guild_id = 300;
        let message_id = 400;
        let channel_id = 500;
        let money = 600;

        database_helpers::create_mock::guild(&data.database, guild_id).await;

        let timestamp_before = Utc::now().timestamp();

        let _ = create_wallet(name.clone(), guild_id, message_id, channel_id, money, &data).await?;
        let timestamp_after = Utc::now().timestamp();

        let wallets = sqlx::query!(
            "SELECT name, guild_id, bot_message_id, bot_message_channel_id, creation_timestamp, money FROM wallet"
        )
            .fetch_all(&data.database)
            .await?;

        let wallet = wallets.first().unwrap();
        assert_eq!(name, wallet.name);
        assert_eq!(guild_id, wallet.guild_id);
        assert_eq!(message_id, wallet.bot_message_id);
        assert_eq!(channel_id, wallet.bot_message_channel_id);
        assert_eq!(money, wallet.money);
        assert_le!(timestamp_before, wallet.creation_timestamp);
        assert_ge!(timestamp_after, wallet.creation_timestamp);

        Ok(())
    }

    #[sqlx::test]
    async fn create_wallet_called_twice_should_fail(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let name = String::from("Test Wallet");
        let guild_id = 300;
        let message_id = 400;
        let channel_id = 500;
        let money = 600;

        database_helpers::create_mock::guild(&data.database, guild_id).await;

        let _ = create_wallet(name.clone(), guild_id, message_id, channel_id, money, &data).await?;

        let result =
            create_wallet(name.clone(), guild_id, message_id, channel_id, money, &data).await;

        assert!(result.is_err());

        Ok(())
    }
}
