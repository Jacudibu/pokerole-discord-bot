use sqlx::{Pool, Sqlite};

use crate::data::Data;
use crate::enums::{Gender, PokemonType, RegionalVariant};
use crate::game_data::pokemon::Pokemon;
use crate::game_data::PokemonApiId;

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

pub async fn get_character_emoji(data: &Data, character_id: i64) -> Option<String> {
    let result = sqlx::query!(
        "SELECT guild_id, species_api_id, is_shiny, phenotype FROM character WHERE id = ?",
        character_id
    )
    .fetch_one(&data.database)
    .await;

    if let Ok(record) = result {
        let gender = Gender::from_phenotype(record.phenotype);
        let api_id = PokemonApiId(record.species_api_id as u16);
        let pokemon = data
            .game_multi_source
            .base_data
            .pokemon_by_api_id
            .get(&api_id)
            .expect("DB species ID should always be set!");

        get_pokemon_emoji(
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

    // Try again, without guild_id. Technically we could just leech emojis off of extra servers
    let result = sqlx::query!("SELECT discord_string FROM emoji WHERE species_api_id = ? AND is_female = ? AND is_shiny = ? AND is_animated = ?", api_id, is_female, is_shiny, is_animated)
        .fetch_one(database)
        .await;

    if let Ok(result) = result {
        return Some(result.discord_string);
    }

    // Any will do! Please!~
    get_any_pokemon_emoji(database, pokemon).await
}

pub async fn get_any_pokemon_emoji(database: &Pool<Sqlite>, pokemon: &Pokemon) -> Option<String> {
    let api_id = pokemon.poke_api_id.0 as i64;

    if pokemon.has_animated_sprite() {
        let result = sqlx::query!(
            "SELECT discord_string FROM emoji WHERE species_api_id = ? AND is_animated = true",
            api_id
        )
        .fetch_one(database)
        .await;

        if let Ok(result) = result {
            return Some(result.discord_string);
        }
    }

    let result = sqlx::query!(
        "SELECT discord_string FROM emoji WHERE species_api_id = ?",
        api_id
    )
    .fetch_one(database)
    .await;

    if let Ok(result) = result {
        return Some(result.discord_string);
    }

    None
}

pub async fn get_any_pokemon_emoji_with_space(
    database: &Pool<Sqlite>,
    pokemon: &Pokemon,
) -> String {
    if let Some(emoji) = get_any_pokemon_emoji(database, pokemon).await {
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
        .replace(['(', ')'], "");

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
