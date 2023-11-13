use crate::cache::{CharacterCacheItem, ShopCacheItem};
use crate::commands::autocompletion::{autocomplete_character_name, autocomplete_shop_name};
use crate::commands::characters::{log_action, validate_user_input, ActionType};
use crate::commands::{
    ensure_guild_exists, find_character, find_shop, send_ephemeral_reply, send_error, Context,
    Error,
};
use crate::data::Data;

/// Adds an existing character as owner for an existing shop.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn add_shop_owner(
    ctx: Context<'_>,
    #[description = "What's the shop's name?"]
    #[autocomplete = "autocomplete_shop_name"]
    shop_name: String,
    #[autocomplete = "autocomplete_character_name"]
    #[description = "Which character should be added as owner?"]
    character_name: String,
) -> Result<(), Error> {
    if let Err(e) = validate_user_input(shop_name.as_str()) {
        return send_error(&ctx, e).await;
    }

    let guild_id = ctx.guild_id().expect("Command is guild_only").0;
    ensure_guild_exists(&ctx, guild_id as i64).await;

    add_shop_owner_impl(ctx.data(), &shop_name, &character_name, guild_id).await?;

    send_ephemeral_reply(
        &ctx,
        format!(
            "Successfully added {} as owner for {}",
            character_name, shop_name
        )
        .as_str(),
    )
    .await?;
    log_action(
        &ActionType::ShopChange,
        &ctx,
        &format!("Added {} as owner for {}", character_name, shop_name,),
    )
    .await?;
    Ok(())
}

async fn add_shop_owner_impl(
    data: &Data,
    shop_name: &str,
    character_name: &str,
    guild_id: u64,
) -> Result<(), Error> {
    let character = find_character(data, guild_id, character_name).await?;
    let shop = find_shop(data, guild_id, shop_name).await?;
    add_shop_owner_to_db(shop, character, data).await?;
    Ok(())
}

async fn add_shop_owner_to_db(
    shop: ShopCacheItem,
    character: CharacterCacheItem,
    data: &Data,
) -> Result<(), String> {
    let result = sqlx::query!(
        "INSERT INTO shop_owner (shop_id, character_id) VALUES (?, ?)",
        shop.id,
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
    use crate::commands::shops::add_shop_owner::add_shop_owner_impl;
    use crate::{database_helpers, Error};
    use sqlx::{Pool, Sqlite};

    #[sqlx::test]
    async fn adding_shop_owner_should_work(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let shop_name = String::from("Test Shop");
        let character_name = String::from("Test Character");
        let guild_id = 100;
        let user_id = 200;
        let shop_id = 300;
        let character_id = 400;

        database_helpers::create_mock::guild(&data.database, guild_id).await;
        database_helpers::create_mock::user(&data.database, user_id).await;
        database_helpers::create_mock::shop(&data.database, guild_id, shop_id, &shop_name).await;
        database_helpers::create_mock::character(
            &data,
            guild_id,
            user_id,
            character_id,
            &character_name,
        )
        .await;

        add_shop_owner_impl(&data, &shop_name, &character_name, guild_id as u64).await?;

        let shop_owners = sqlx::query!("SELECT shop_id, character_id FROM shop_owner")
            .fetch_all(&data.database)
            .await?;

        let shop_owner = shop_owners.first().unwrap();
        assert_eq!(character_id, shop_owner.character_id);
        assert_eq!(shop_id, shop_owner.shop_id);

        Ok(())
    }

    #[sqlx::test]
    async fn add_shop_owner_called_twice_should_fail(db: Pool<Sqlite>) -> Result<(), Error> {
        let data = database_helpers::create_mock::data(db).await;
        let shop_name = String::from("Test Shop");
        let character_name = String::from("Test Character");
        let guild_id = 100;
        let user_id = 200;
        let shop_id = 300;
        let character_id = 400;

        database_helpers::create_mock::guild(&data.database, guild_id).await;
        database_helpers::create_mock::user(&data.database, user_id).await;
        database_helpers::create_mock::shop(&data.database, guild_id, shop_id, &shop_name).await;
        database_helpers::create_mock::character(
            &data,
            guild_id,
            user_id,
            character_id,
            &character_name,
        )
        .await;

        add_shop_owner_impl(&data, &shop_name, &character_name, guild_id as u64).await?;
        let result = add_shop_owner_impl(&data, &shop_name, &character_name, guild_id as u64).await;

        assert!(result.is_err());

        Ok(())
    }
}
