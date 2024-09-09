use serenity::all::{CreateMessage, GetMessages};
use serenity::model::user::User;

use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::characters::{log_action, validate_user_input, ActionType};
use crate::commands::{
    create_emojis, ensure_guild_exists, ensure_user_exists, pokemon_from_autocomplete_string,
    send_ephemeral_reply, send_error, update_character_post, Context, Error,
};
use crate::enums::Gender;
use crate::{emoji, helpers};

/// Create a new character within the database.
#[allow(clippy::too_many_arguments)]
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn create_character(
    ctx: Context<'_>,
    #[description = "Who owns the character?"] player: User,
    #[description = "What's the character's name?"] name: String,
    #[autocomplete = "autocomplete_pokemon"]
    #[description = "What kind of pokemon are you?"]
    pokemon_species: String,
    #[description = "Optional. Does it glow in the dark? Defaults to false."] is_shiny: Option<
        bool,
    >,
    #[description = "Which phenotype?"] gender: Gender,
    #[description = "Optional. Defaults to 0."]
    #[min = 0_i64]
    exp: Option<i64>,
    #[description = "Optional. Defaults to 500."]
    #[min = 0_i64]
    money: Option<i64>,
) -> Result<(), Error> {
    execute(
        ctx,
        player,
        name,
        pokemon_species,
        is_shiny,
        gender,
        exp,
        money,
    )
    .await
}

/// Deprecated. Use create_character instead.
#[allow(clippy::too_many_arguments)]
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
    #[description = "Which phenotype?"] gender: Gender,
    #[description = "Optional. Defaults to 0."]
    #[min = 0_i64]
    exp: Option<i64>,
    #[description = "Optional. Defaults to 500."]
    #[min = 0_i64]
    money: Option<i64>,
) -> Result<(), Error> {
    execute(
        ctx,
        player,
        name,
        pokemon_species,
        is_shiny,
        gender,
        exp,
        money,
    )
    .await
}

async fn execute(
    ctx: Context<'_>,
    player: User,
    name: String,
    pokemon_species: String,
    is_shiny: Option<bool>,
    gender: Gender,
    exp: Option<i64>,
    money: Option<i64>,
) -> Result<(), Error> {
    if let Err(e) = validate_user_input(name.as_str()) {
        return send_error(&ctx, e).await;
    }

    let pokemon = pokemon_from_autocomplete_string(&ctx, &pokemon_species).await?;
    let is_shiny = is_shiny.unwrap_or(false);
    let exp = exp.unwrap_or(0);
    let money = money.unwrap_or(500);
    let phenotype = gender as i64;

    let message = ctx
        .channel_id()
        .send_message(
            ctx,
            CreateMessage::new().content(
                "[Placeholder. This should get replaced or deleted within a couple seconds.]",
            ),
        )
        .await?;

    if let Ok(messages) = ctx.channel_id().messages(ctx, GetMessages::new()).await {
        if messages.len() > 10 {
            send_error(&ctx, "Seems like there's already been a looot of messages been sent in this channel! Cannot create a character here, that seems wrong.").await?;
            message.delete(ctx).await?;
            return Ok(());
        }
    }

    let user_id = player.id.get() as i64;
    let guild_id = ctx.guild_id().expect("Command is guild_only").get() as i64;

    ensure_guild_exists(&ctx, guild_id).await;
    ensure_user_exists(&ctx, user_id, guild_id).await;

    let stat_message_id = message.id.get() as i64;
    let stat_channel_id = message.channel_id.get() as i64;
    let creation_date = chrono::Utc::now().date_naive();

    let level = helpers::calculate_level_from_experience(exp);
    let mon = helpers::get_usual_evolution_stage_for_level(
        level,
        pokemon,
        &ctx.data().game.base_data,
        None,
    );

    let record = sqlx::query!(
        "INSERT INTO character (user_id, guild_id, name, stat_message_id, stat_channel_id, creation_date, experience, money, species_api_id, is_shiny, phenotype,\
                                stat_strength, stat_dexterity, stat_vitality, stat_special, stat_insight
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        user_id,
        guild_id,
        name,
        stat_message_id,
        stat_channel_id,
        creation_date,
        exp,
        money,
        pokemon.poke_api_id.0,
        is_shiny,
        phenotype,
        mon.strength.min,
        mon.dexterity.min,
        mon.vitality.min,
        mon.special.min,
        mon.insight.min,
    ).fetch_one(&ctx.data().database)
        .await;

    create_emojis::create_emojis_for_pokemon(&ctx, pokemon, &gender, is_shiny).await;

    if let Ok(record) = record {
        send_ephemeral_reply(&ctx, "Character has been successfully created!").await?;
        update_character_post(&ctx, record.id).await;
        log_action(
            &ActionType::Initialization,
            &ctx,
            &format!(
                "Initialized character {} ({}) with {} {} and {} exp.",
                name,
                pokemon.name,
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
