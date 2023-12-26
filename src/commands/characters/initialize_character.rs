use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::characters::{
    log_action, update_character_post, validate_user_input, ActionType,
};
use crate::commands::{
    ensure_guild_exists, ensure_user_exists, send_ephemeral_reply, send_error, Context, Error,
};
use crate::emoji;
use serenity::all::CreateMessage;
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
    #[autocomplete = "autocomplete_pokemon"]
    #[description = "What kind of pokemon are you?"]
    pokemon_species: String,
    #[description = "Optional. Does it glow in the dark? Defaults to false."] is_shiny: Option<
        bool,
    >,
    #[description = "Optional. Defaults to 0."]
    #[min = 0_i64]
    exp: Option<i64>,
    #[description = "Optional. Defaults to 500."]
    #[min = 0_i64]
    money: Option<i64>,
) -> Result<(), Error> {
    if let Err(e) = validate_user_input(name.as_str()) {
        return send_error(&ctx, e).await;
    }

    let pokemon = ctx.data().game.pokemon.get(&pokemon_species.to_lowercase());
    if pokemon.is_none() {
        return send_error(&ctx, &std::format!("Unable to find a pokemon named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", pokemon_species)).await;
    }
    let pokemon = pokemon.expect("This was just checked.");

    let is_shiny = is_shiny.unwrap_or(false);
    let exp = exp.unwrap_or(0);
    let money = money.unwrap_or(500);

    let message = ctx
        .channel_id()
        .send_message(
            ctx,
            CreateMessage::new().content(
                "[Placeholder. This should get replaced or deleted within a couple seconds.]",
            ),
        )
        .await?;

    let user_id = player.id.get() as i64;
    let guild_id = ctx.guild_id().expect("Command is guild_only").get() as i64;

    ensure_guild_exists(&ctx, guild_id).await;
    ensure_user_exists(&ctx, user_id, guild_id).await;

    let stat_message_id = message.id.get() as i64;
    let stat_channel_id = message.channel_id.get() as i64;
    let creation_date = chrono::Utc::now().date_naive();

    let record = sqlx::query!(
        "INSERT INTO character (user_id, guild_id, name, stat_message_id, stat_channel_id, creation_date, experience, money, species_api_id, is_shiny) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        user_id,
        guild_id,
        name,
        stat_message_id,
        stat_channel_id,
        creation_date,
        exp,
        money,
        pokemon.poke_api_id.0,
        is_shiny
    ).fetch_one(&ctx.data().database)
        .await;

    if let Ok(record) = record {
        send_ephemeral_reply(&ctx, "Character has been successfully created!").await?;
        update_character_post(&ctx, record.id).await;
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
