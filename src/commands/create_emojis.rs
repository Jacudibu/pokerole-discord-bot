use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{
    ensure_guild_exists, pokemon_from_autocomplete_string, send_ephemeral_reply, send_error,
};
use crate::shared::emoji::EmojiData;
use crate::shared::enums::Gender;
use crate::shared::game_data::pokemon::Pokemon;
use crate::shared::{emoji, PoiseContext};
use crate::Error;
use log::info;
use serenity::all::{CreateAttachment, Emoji, GuildId};
use sqlx::{Pool, Sqlite};

async fn upload_emoji_to_guild<'a>(
    ctx: &PoiseContext<'a>,
    emoji_data: EmojiData,
) -> Result<(GuildId, Emoji), serenity::all::Error> {
    let guild_id = ctx.guild_id().expect("create_emoji Command is guild_only!");
    let attachment = CreateAttachment::bytes(emoji_data.data, &emoji_data.name);
    match guild_id
        .create_emoji(&ctx, emoji_data.name.as_str(), &attachment.to_base64())
        .await
    {
        Ok(emoji) => {
            let _ = send_ephemeral_reply(ctx, &format!("Created new emoji: {}", emoji)).await;
            Ok((guild_id, emoji))
        }
        Err(e) => {
            // Server is probably at emoji capacity. Too bad!
            Err(e)
        }
    }
}

/// Creates new emojis!
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn create_emojis(
    ctx: PoiseContext<'_>,
    #[description = "Which pokemon?"]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
    #[description = "Which phenotype?"] gender: Gender,
    #[description = "Does it glow in the dark?"] is_shiny: bool,
) -> Result<(), Error> {
    let pokemon = pokemon_from_autocomplete_string(&ctx, &name).await?;
    let created_emojis = create_emojis_for_pokemon(&ctx, pokemon, &gender, is_shiny).await;
    if created_emojis == 0 {
        let _ = send_error(&ctx, "Emojis for this pokemon already seem to exist!").await;
    }

    Ok(())
}

pub async fn create_emojis_for_pokemon<'a>(
    ctx: &PoiseContext<'a>,
    pokemon: &Pokemon,
    gender: &Gender,
    is_shiny: bool,
) -> u8 {
    let guild_id = ctx.guild_id().expect("Emoji creation is guild_only.").get() as i64;
    let mut created_emojis = 0u8;
    if !does_emoji_exist_in_database(
        &ctx.data().database,
        guild_id,
        pokemon,
        gender,
        is_shiny,
        false,
    )
    .await
    {
        create_emoji_and_notify_user(ctx, pokemon, gender, is_shiny, false).await;
        created_emojis += 1u8;
    }

    if pokemon.has_animated_sprite()
        && !does_emoji_exist_in_database(
            &ctx.data().database,
            guild_id,
            pokemon,
            gender,
            is_shiny,
            true,
        )
        .await
    {
        create_emoji_and_notify_user(ctx, pokemon, gender, is_shiny, true).await;
        created_emojis += 1u8;
    }

    created_emojis
}

pub async fn store_emoji_in_database(
    database: &Pool<Sqlite>,
    guild_id: GuildId,
    emoji: &Emoji,
    pokemon: &Pokemon,
    gender: &Gender,
    is_shiny: bool,
    is_animated: bool,
) {
    let guild_id = guild_id.get() as i64;
    let api_id = pokemon.poke_api_id.0 as i64;
    let is_female = pokemon.species_data.has_gender_differences && gender == &Gender::Female;
    let discord_string = emoji.to_string();
    match sqlx::query!("INSERT INTO emoji (species_api_id, guild_id, is_female, is_shiny, is_animated, discord_string) VALUES (?, ?, ?, ?, ?, ?)", api_id, guild_id, is_female, is_shiny, is_animated, discord_string).execute(database).await {
        Ok(_) => {}
        Err(e) => {info!("{:?}", e);}
    };
}

pub async fn does_emoji_exist_in_database(
    database: &Pool<Sqlite>,
    guild_id: i64,
    pokemon: &Pokemon,
    gender: &Gender,
    is_shiny: bool,
    is_animated: bool,
) -> bool {
    let api_id = pokemon.poke_api_id.0 as i64;
    let is_female = pokemon.species_data.has_gender_differences && gender == &Gender::Female;

    let result = sqlx::query!("SELECT COUNT(*) as count FROM emoji WHERE species_api_id = ? AND guild_id = ? AND is_female = ? AND is_shiny = ? AND is_animated = ?", api_id, guild_id, is_female, is_shiny, is_animated)
        .fetch_one(database)
        .await;

    if let Ok(result) = result {
        result.count > 0
    } else {
        false
    }
}

async fn create_emoji_and_notify_user<'a>(
    ctx: &PoiseContext<'a>,
    pokemon: &Pokemon,
    gender: &Gender,
    is_shiny: bool,
    is_animated: bool,
) {
    let guild_id = ctx.guild_id().expect("Creating emoji is guild_only!").get() as i64;
    ensure_guild_exists(ctx, guild_id).await;

    match emoji::get_emoji_data(pokemon, gender, is_shiny, is_animated) {
        Ok(emoji_data) => match upload_emoji_to_guild(ctx, emoji_data).await {
            Ok((guild_id, emoji)) => {
                store_emoji_in_database(
                    &ctx.data().database,
                    guild_id,
                    &emoji,
                    pokemon,
                    gender,
                    is_shiny,
                    is_animated,
                )
                .await;
            }
            Err(e) => {
                let _ = send_error(
                    ctx,
                    &format!(
                        "Something went wrong when uploading the emoji to discord: {:?}",
                        e
                    ),
                )
                .await;
            }
        },
        Err(e) => {
            let _ = send_error(
                ctx,
                &format!("Something went wrong when parsing the emoji: {:?}", e),
            )
            .await;
        }
    }
}
