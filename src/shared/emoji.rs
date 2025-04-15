use crate::shared::data::Data;
use crate::shared::enums::{Gender, PokemonType, RegionalVariant};
use crate::shared::game_data::pokemon::Pokemon;
use crate::shared::game_data::PokemonApiId;
use crate::shared::{constants, emoji, helpers};
use crate::Error;
use image::{DynamicImage, GenericImageView, ImageFormat};
use rand::Rng;
use serenity::all::{CreateAttachment, CreateMessage, Emoji};
use sqlx::{Pool, Sqlite};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};

pub const POKE_COIN: &str = "<:poke_coin:1120237132200546304>";
pub const BATTLE_POINT: &str = "<:battle_point:1202570025802661938>";

pub const RANK_BRONZE: &str = "<:badge_bronze:1119186018793435177>";
pub const RANK_SILVER: &str = "<:badge_silver:1119185975545954314>";
pub const RANK_GOLD: &str = "<:badge_gold:1119185974149251092>";
pub const RANK_PLATINUM: &str = "<:badge_platinum:1119185972635107378>";
pub const RANK_DIAMOND: &str = "<:badge_diamond:1119185970374389760>";

pub const UNICODE_CROSS_MARK_BUTTON: &str = "❎";
pub const UNICODE_CROSS_MARK: &str = "❌";
pub const UNICODE_CHECK_MARK: &str = "✔️";

pub const TROPHY: &str = "🏆";
pub const BACKPACK: &str = "🎒";
pub const FENCING: &str = "🤺";
pub const TICKET: &str = "🎫";
pub const CROSSED_SWORDS: &str = "⚔️";
pub const PARTY_POPPER: &str = "🎉";
pub const PARTYING_FACE: &str = "🥳";

pub const DOT_EMPTY: char = '⭘';
pub const DOT_FILLED: char = '⬤';
pub const DOT_OVERCHARGED: char = '⧳';

pub async fn get_character_emoji(
    context: &serenity::all::Context,
    data: &Data,
    character_id: i64,
) -> Option<String> {
    let result = sqlx::query!(
        "SELECT guild_id, species_api_id, is_shiny, phenotype FROM character WHERE id = ?",
        character_id
    )
    .fetch_one(&data.database)
    .await;

    if let Ok(record) = result {
        let gender = Gender::from_phenotype(record.phenotype);
        let api_id = PokemonApiId(record.species_api_id as u16);
        let Some(pokemon) = data.game.base_data.pokemon_by_api_id.get(&api_id) else {
            helpers::log_error(context, format!("DB species ID should always be set, but was unable to find a pokemon for api_id {api_id:?}!")).await;
            return None;
        };

        get_pokemon_emoji(
            context,
            &data.database,
            record.guild_id,
            pokemon,
            &gender,
            record.is_shiny,
        )
        .await
    } else {
        None
    }
}

pub async fn get_pokemon_emoji(
    context: &serenity::all::Context,
    database: &Pool<Sqlite>,
    guild_id: i64,
    pokemon: &Pokemon,
    gender: &Gender,
    is_shiny: bool,
) -> Option<String> {
    let api_id = pokemon.poke_api_id.0 as i64;
    let is_female = pokemon.species_data.has_gender_differences && gender == &Gender::Female;
    let is_animated = pokemon.has_animated_sprite();

    let result = sqlx::query!("SELECT discord_string FROM emoji WHERE species_api_id = ? AND guild_id = ? AND is_female = ? AND is_shiny = ? AND is_animated = ?", api_id, guild_id, is_female, is_shiny, is_animated)
        .fetch_one(database)
        .await;

    if let Ok(result) = result {
        return Some(result.discord_string);
    }

    get_application_emoji(context, database, pokemon).await
}

pub async fn get_application_emoji(
    context: &serenity::all::Context,
    database: &Pool<Sqlite>,
    pokemon: &Pokemon,
) -> Option<String> {
    let api_id = pokemon.poke_api_id.0 as i64;
    let result = sqlx::query!(
        "SELECT discord_string FROM application_emoji WHERE species_api_id = ?",
        api_id
    )
    .fetch_one(database)
    .await;

    match result {
        Ok(result) => Some(result.discord_string),
        Err(_) => match create_application_emoji(context, database, pokemon).await {
            Ok(result) => Some(result.to_string()),
            Err(_) => None,
        },
    }
}

pub async fn get_any_pokemon_emoji_with_space(
    context: &serenity::all::Context,
    database: &Pool<Sqlite>,
    pokemon: &Pokemon,
) -> String {
    if let Some(emoji) = get_application_emoji(context, database, pokemon).await {
        format!("{} ", emoji)
    } else {
        String::new()
    }
}

pub fn pokemon_to_emoji_name(
    pokemon: &Pokemon,
    is_female: bool,
    is_shiny: bool,
    is_animated: bool,
) -> String {
    let shiny = if is_shiny { "shiny_" } else { "" };
    let female = if is_female { "_female" } else { "" };
    let mut name = pokemon
        .name
        .to_lowercase()
        .replace([' ', '-'], "_")
        .replace(['(', ')', '.', '\''], "");

    let regional_prefix = if let Some(regional_variant) = pokemon.regional_variant {
        name = name
            .replace("paldean_form", "")
            .replace("hisuian_form", "")
            .replace("galarian_form", "")
            .replace("alolan_form", "");

        match regional_variant {
            RegionalVariant::Alola => "alolan_",
            RegionalVariant::Galar => "galarian",
            RegionalVariant::Hisui => "hisuian_",
            RegionalVariant::Paldea => "paldean_",
        }
    } else {
        ""
    };

    let animated = if is_animated { "_animated" } else { "" };

    format!(
        "{}{}{}{}{}",
        shiny,
        regional_prefix,
        name.trim_matches('_'),
        female,
        animated
    )
}

pub fn type_to_emoji(pokemon_type: &PokemonType) -> &str {
    match pokemon_type {
        PokemonType::Normal => "<:type_normal:1118590014931095662>",
        PokemonType::Fighting => "<:type_fighting:1118590013194649730>",
        PokemonType::Flying => "<:type_flying:1118590010359283773>",
        PokemonType::Poison => "<:type_poison:1118590008778047529>",
        PokemonType::Ground => "<:type_ground:1118590006081114325>",
        PokemonType::Rock => "<:type_rock:1118590005082861820>",
        PokemonType::Bug => "<:type_bug:1118594892566908959>",
        PokemonType::Ghost => "<:type_ghost:1118594890461368350>",
        PokemonType::Steel => "<:type_steel:1118594889131765821>",
        PokemonType::Fire => "<:type_fire:1118594887399514145>",
        PokemonType::Water => "<:type_water:1118594885344297062>",
        PokemonType::Grass => "<:type_grass:1118594883754664107>",
        PokemonType::Electric => "<:type_electric:1118594871272415243>",
        PokemonType::Psychic => "<:type_psychic:1118594873755435009>",
        PokemonType::Ice => "<:type_ice:1118594875085041825>",
        PokemonType::Dragon => "<:type_dragon:1118594876444000357>",
        PokemonType::Dark => "<:type_dark:1118594879195447387>",
        PokemonType::Fairy => "<:type_fairy:1118594881368100894>",
        PokemonType::Shadow => "",
    }
}

pub struct EmojiData {
    pub data: Vec<u8>,
    pub name: String,
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

pub fn get_emoji_data(
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
            name: pokemon_to_emoji_name(pokemon, use_female_sprite, is_shiny, is_animated),
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
        name: pokemon_to_emoji_name(pokemon, use_female_sprite, is_shiny, is_animated),
    })
}

pub async fn upload_emoji_to_application(
    ctx: &serenity::all::Context,
    emoji_data: EmojiData,
) -> Result<Emoji, serenity::all::Error> {
    let attachment = CreateAttachment::bytes(emoji_data.data, &emoji_data.name);
    match ctx
        .create_application_emoji(&emoji_data.name, &attachment.to_base64())
        .await
    {
        Ok(emoji) => Ok(emoji),
        Err(e) => {
            let _ = constants::ERROR_LOG_CHANNEL
                .send_message(
                    &ctx,
                    CreateMessage::new()
                        .content(format!("Failed to create Application Emoji: {:?}", e)),
                )
                .await;

            Err(e)
        }
    }
}

pub async fn create_application_emoji<'a>(
    ctx: &serenity::all::Context,
    database: &Pool<Sqlite>,
    pokemon: &Pokemon,
) -> Result<Emoji, String> {
    let gender = if pokemon.species_data.has_gender_differences {
        if rand::rng().random_bool(0.5) {
            Gender::Female
        } else {
            Gender::Male
        }
    } else {
        Gender::Male
    };

    match get_emoji_data(pokemon, &gender, false, pokemon.has_animated_sprite()) {
        Ok(emoji_data) => {
            match upload_emoji_to_application(ctx, emoji_data).await {
                Ok(emoji) => {
                    let api_id = pokemon.poke_api_id.0 as i64;
                    let discord_string = emoji.to_string();

                    let _ = sqlx::query!("INSERT into application_emoji (species_api_id, discord_string) VALUES (?, ?)", api_id, discord_string).
                        execute(database)
                        .await;

                    Ok(emoji)
                }
                Err(e) => {
                    let message = format!(
                        "Unable to upload emoji for pokemon {} with id {} to application: {:?}",
                        pokemon.name, pokemon.poke_api_id.0, e
                    );
                    let _ = constants::ERROR_LOG_CHANNEL
                        .send_message(
                            ctx,
                            CreateMessage::new()
                                .content(format!("Unable to upload emoji for pokemon {} with id {} to application: {:?}", pokemon.name, pokemon.poke_api_id.0, e)),
                        )
                        .await;

                    Err(message)
                }
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
