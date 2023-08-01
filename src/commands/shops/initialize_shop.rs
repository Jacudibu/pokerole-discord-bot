use crate::commands::characters::{log_action, validate_user_input, ActionType};
use crate::commands::shops::update_shop_post;
use crate::commands::{ensure_guild_exists, send_ephemeral_reply, send_error, Context, Error};
use crate::data::Data;
use crate::emoji;
use chrono::Utc;

/// Create a new shop within the database.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn initialize_shop(
    ctx: Context<'_>,
    #[description = "What's the shop's name?"] name: String,
    #[min = 0_i64] money: i64,
) -> Result<(), Error> {
    if let Err(e) = validate_user_input(name.as_str()) {
        return send_error(&ctx, e).await;
    }

    let message = ctx
        .channel_id()
        .send_message(ctx, |f| {
            f.content("[Placeholder. This should get replaced or deleted within a couple seconds.]")
        })
        .await?;

    let guild_id = ctx.guild_id().expect("Command is guild_only").0 as i64;

    ensure_guild_exists(&ctx, guild_id).await;

    let message_id = message.id.0 as i64;
    let channel_id = message.channel_id.0 as i64;

    let result = create_shop(
        name.clone(),
        guild_id,
        message_id,
        channel_id,
        money,
        ctx.data(),
    )
    .await;
    if let Ok(id) = result {
        send_ephemeral_reply(&ctx, "Shop has been successfully created!").await?;
        update_shop_post(&ctx, id).await;
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
        "Something went wrong! Does a shop with this name already exist on this server?",
    )
    .await?;
    message.delete(ctx).await?;

    Ok(())
}

async fn create_shop(
    name: String,
    guild_id: i64,
    message_id: i64,
    channel_id: i64,
    money: i64,
    data: &Data,
) -> Result<i64, String> {
    let timestamp = Utc::now().timestamp();

    let result = sqlx::query!(
        "INSERT INTO shop (name, guild_id, bot_message_id, bot_message_channel_id, creation_timestamp, money) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
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
    use crate::commands::shops::initialize_shop::create_shop;
    use crate::{database_helpers, Error};
    use chrono::Utc;
    use more_asserts::{assert_ge, assert_le};
    use sqlx::{Pool, Sqlite};

    #[sqlx::test]
    async fn create_shop_should_work(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let name = String::from("Test Shop");
        let guild_id = 300;
        let message_id = 400;
        let channel_id = 500;
        let money = 600;

        database_helpers::create_mock::guild(&data.database, guild_id).await;

        let timestamp_before = Utc::now().timestamp();

        let _ = create_shop(name.clone(), guild_id, message_id, channel_id, money, &data).await?;
        let timestamp_after = Utc::now().timestamp();

        let shops = sqlx::query!(
            "SELECT name, guild_id, bot_message_id, bot_message_channel_id, creation_timestamp, money FROM shop"
        )
            .fetch_all(&data.database)
            .await?;

        let shop = shops.first().unwrap();
        assert_eq!(name, shop.name);
        assert_eq!(guild_id, shop.guild_id);
        assert_eq!(message_id, shop.bot_message_id);
        assert_eq!(channel_id, shop.bot_message_channel_id);
        assert_eq!(money, shop.money);
        assert_le!(timestamp_before, shop.creation_timestamp);
        assert_ge!(timestamp_after, shop.creation_timestamp);

        Ok(())
    }

    #[sqlx::test]
    async fn create_shop_called_twice_should_fail(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let name = String::from("Test Shop");
        let guild_id = 300;
        let message_id = 400;
        let channel_id = 500;
        let money = 600;

        database_helpers::create_mock::guild(&data.database, guild_id).await;

        let _ = create_shop(name.clone(), guild_id, message_id, channel_id, money, &data).await?;

        let result =
            create_shop(name.clone(), guild_id, message_id, channel_id, money, &data).await;

        assert!(result.is_err());

        Ok(())
    }
}
