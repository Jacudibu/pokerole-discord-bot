use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{
    ensure_guild_exists, pokemon_from_autocomplete_string, send_ephemeral_reply, send_error,
    Context,
};
use crate::enums::Gender;
use crate::game_data::pokemon::Pokemon;
use crate::{emoji, Error};
use image::{DynamicImage, GenericImageView, ImageFormat};
use log::info;
use serenity::all::{CreateAttachment, Emoji, GuildId};
use sqlx::{Pool, Sqlite};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};

fn find_top_border(image: &DynamicImage) -> u32 {
    for y in 0..image.height() {
        let mut contains_something = false;
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y);
            if pixel.0[3] > 0 {
                contains_something = true;
                break;
            }
        }

        if contains_something {
            return y;
        }
    }

    0
}

fn find_bottom_border(image: &DynamicImage) -> u32 {
    for y in (0..image.height()).rev() {
        let mut contains_something = false;
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y);
            if pixel.0[3] > 0 {
                contains_something = true;
                break;
            }
        }

        if contains_something {
            return y + 1;
        }
    }

    image.height()
}

fn find_left_border(image: &DynamicImage) -> u32 {
    for x in 0..image.width() {
        let mut contains_something = false;
        for y in 0..image.height() {
            let pixel = image.get_pixel(x, y);
            if pixel.0[3] > 0 {
                contains_something = true;
                break;
            }
        }

        if contains_something {
            return x;
        }
    }

    0
}

fn find_right_border(image: &DynamicImage) -> u32 {
    for x in (0..image.width()).rev() {
        let mut contains_something = false;
        for y in 0..image.height() {
            let pixel = image.get_pixel(x, y);
            if pixel.0[3] > 0 {
                contains_something = true;
                break;
            }
        }

        if contains_something {
            return x + 1;
        }
    }

    image.width()
}

struct EmojiData {
    data: Vec<u8>,
    name: String,
}

fn local_emoji_path(
    pokemon: &Pokemon,
    is_female: bool,
    is_shiny: bool,
    is_animated: bool,
) -> String {
    let path = std::env::var("POKEMON_API_SPRITES")
        .expect("missing POKEMON_API_SPRITES environment variable.");

    let animated_path = if is_animated {
        "versions/generation-v/black-white/animated/"
    } else {
        ""
    };
    let shiny_path = if is_shiny { "shiny/" } else { "" };
    let gender_path = if is_female { "female/" } else { "" };
    let file_type = if is_animated { "gif" } else { "png" };

    format!(
        "{}sprites/pokemon/{}{}{}{}.{}",
        path, animated_path, shiny_path, gender_path, pokemon.poke_api_id.0, file_type
    )
}

fn get_emoji_data(
    pokemon: &Pokemon,
    gender: &Gender,
    is_shiny: bool,
    is_animated: bool,
) -> Result<EmojiData, Error> {
    let use_female_sprite =
        pokemon.species_data.has_gender_differences && gender == &Gender::Female;

    let path = local_emoji_path(pokemon, use_female_sprite, is_shiny, is_animated);
    if is_animated {
        let mut file = File::open(path)?;
        let mut out = Vec::new();
        file.read_to_end(&mut out)?;
        return Ok(EmojiData {
            data: out,
            name: emoji::pokemon_to_emoji_name(pokemon, use_female_sprite, is_shiny, is_animated),
        });
    }

    let image = crop_whitespace(image::open(path)?);
    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::Png)?;

    cursor.rewind()?;
    let reader = &mut BufReader::new(&mut cursor);
    let mut out = Vec::new();
    reader.read_to_end(&mut out)?;

    Ok(EmojiData {
        data: out,
        name: emoji::pokemon_to_emoji_name(pokemon, use_female_sprite, is_shiny, is_animated),
    })
}

fn crop_whitespace(image: DynamicImage) -> DynamicImage {
    let mut top_border = find_top_border(&image);
    let bottom_border = find_bottom_border(&image);
    let left_border = find_left_border(&image);
    let right_border = find_right_border(&image);

    let height = bottom_border - top_border;
    let width = right_border - left_border;
    if height < width {
        // Make it square and move the cutout towards the bottom so the mon "stands" on the ground.
        let diff = width - height;
        if (top_border as i32) - (diff as i32) < 0 {
            top_border = 0;
        } else {
            top_border -= diff;
        }
    }

    image.crop_imm(left_border, top_border, width, bottom_border - top_border)
}

async fn upload_emoji_to_server_or_emoji_guild<'a>(
    ctx: &Context<'a>,
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
            // Server is probably at emoji capacity, upload to emoji server instead.
            match sqlx::query!("SELECT id, emoji_count FROM emoji_guild ORDER BY emoji_count DESC")
                .fetch_one(&ctx.data().database)
                .await
            {
                Ok(record) => {
                    let guild_id = GuildId::new(record.id as u64);
                    match guild_id
                        .create_emoji(&ctx, emoji_data.name.as_str(), &attachment.to_base64())
                        .await
                    {
                        Ok(emoji) => {
                            let _ = send_ephemeral_reply(
                                ctx,
                                &format!(
                                    "\
Created new emoji: {}\n\
This server has reached its emoji capacity, but I won't be stopped by such trivial things!",
                                    emoji
                                ),
                            )
                            .await;

                            let new_count = record.emoji_count + 1;
                            let _ = sqlx::query!(
                                "UPDATE emoji_guild SET emoji_count = ? WHERE id = ?",
                                new_count,
                                record.id
                            )
                            .execute(&ctx.data().database)
                            .await;

                            Ok((guild_id, emoji))
                        }
                        Err(e) => Err(e),
                    }
                }
                Err(_) => Err(e),
            }
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
    ctx: Context<'_>,
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
    ctx: &Context<'a>,
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
    ctx: &Context<'a>,
    pokemon: &Pokemon,
    gender: &Gender,
    is_shiny: bool,
    is_animated: bool,
) {
    let guild_id = ctx.guild_id().expect("Creating emoji is guild_only!").get() as i64;
    ensure_guild_exists(ctx, guild_id).await;

    match get_emoji_data(pokemon, gender, is_shiny, is_animated) {
        Ok(emoji_data) => match upload_emoji_to_server_or_emoji_guild(ctx, emoji_data).await {
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
