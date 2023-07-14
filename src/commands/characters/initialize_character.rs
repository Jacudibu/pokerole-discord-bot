use crate::commands::characters::{
    log_action, update_character_post, validate_user_input, ActionType,
};
use crate::commands::{send_ephemeral_reply, send_error, Context, Error};
use crate::emoji;
use serenity::model::id::{GuildId, UserId};
use serenity::model::user::User;

/// Create a new character within the database.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn initialize_character(
    ctx: Context<'_>,
    #[description = "Who owns the character?"] player: User,
    #[description = "What's the character's name?"] name: String,
    #[min = 0_i64] exp: i64,
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

    let user_id = player.id.0 as i64;
    let guild_id = ctx.guild_id().expect("Command is guild_only").0 as i64;

    ensure_guild_exists(&ctx, guild_id).await;
    ensure_user_exists(&ctx, user_id, guild_id).await;

    let stat_message_id = message.id.0 as i64;
    let stat_channel_id = message.channel_id.0 as i64;
    let creation_date = chrono::Utc::now().date_naive();

    let record = sqlx::query!(
        "INSERT INTO character (user_id, guild_id, name, stat_message_id, stat_channel_id, creation_date, experience, money) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        user_id,
        guild_id,
        name,
        stat_message_id,
        stat_channel_id,
        creation_date,
        exp,
        money
    ).fetch_one(&ctx.data().database)
        .await;

    if let Ok(record) = record {
        send_ephemeral_reply(&ctx, "Character has been successfully created!").await?;
        update_character_post(&ctx, record.id).await?;
        log_action(
            &ActionType::Initialization,
            &ctx,
            &format!(
                "Initialized character {} with {} {} and {} exp.",
                name,
                money,
                emoji::POKE_COIN,
                exp
            ),
        )
        .await?;
        ctx.data()
            .cache
            .update_character_names(&ctx.data().database)
            .await;
        return Ok(());
    }

    send_error(&ctx, "Something went wrong! Does a character with this name already exist for this specific player?").await?;
    message.delete(ctx).await?;

    Ok(())
}

async fn ensure_guild_exists<'a>(ctx: &Context<'a>, guild_id: i64) {
    let _ = sqlx::query!("INSERT OR IGNORE INTO guild (id) VALUES (?)", guild_id)
        .execute(&ctx.data().database)
        .await;
}

async fn ensure_user_exists<'a>(ctx: &Context<'a>, user_id: i64, guild_id: i64) {
    let _ = sqlx::query!("INSERT OR IGNORE INTO user (id) VALUES (?)", user_id)
        .execute(&ctx.data().database)
        .await;

    let user = UserId::from(user_id as u64).to_user(ctx).await;
    if let Ok(user) = user {
        let nickname = user
            .nick_in(ctx, GuildId::from(guild_id as u64))
            .await
            .unwrap_or(user.name.clone());
        let _ = sqlx::query!(
            "INSERT OR IGNORE INTO user_in_guild (user_id, guild_id, name) VALUES (?, ?, ?)",
            user_id,
            guild_id,
            nickname
        )
        .execute(&ctx.data().database)
        .await;
    }
}
