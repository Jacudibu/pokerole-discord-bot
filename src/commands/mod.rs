use crate::cache::{CharacterCacheItem, ShopCacheItem};
use crate::data::Data;
use crate::parse_error::ParseError;
use crate::Error;
use poise::{Command, ReplyHandle};
use serenity::model::id::{GuildId, UserId};
use serenity::model::prelude::User;

type Context<'a> = poise::Context<'a, Data, Error>;

mod autocompletion;

pub mod ability;
pub mod about;
pub mod calculate_hp_damage_modifier;
mod create_role_reaction_post;
pub mod efficiency;
pub mod emoji;
pub mod encounter;
pub mod item;
pub mod learns;
pub mod metronome;
pub mod r#move;
pub mod nature;
pub mod poll;
pub mod potion;
pub mod roll;
pub mod rule;
pub mod scale;
pub mod select_random;
pub mod stats;
pub mod status;
pub mod timestamp;
pub mod weather;

mod characters;
mod quests;
mod say;
mod setting_time_offset;
mod shops;
mod update_user_names;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    let mut result = vec![
        roll::roll(),
        roll::r(),
        r#move::poke_move(),
        ability::ability(),
        item::item(),
        stats::stats(),
        stats::pokemon(),
        status::status(),
        rule::rule(),
        learns::learns(),
        nature::nature(),
        timestamp::timestamp(),
        weather::weather(),
        metronome::metronome(),
        efficiency::efficiency(),
        select_random::select_random(),
        poll::poll(),
        scale::scale(),
        emoji::emoji(),
        encounter::encounter(),
        potion::potion(),
        calculate_hp_damage_modifier::calculate_hp_damage_modifier(),
        create_role_reaction_post::create_role_reaction_post(),
        setting_time_offset::setting_time_offset(),
        say::say(),
        update_user_names::update_user_names(),
        about::about(),
    ];

    for x in characters::get_all_commands() {
        result.push(x);
    }
    for x in shops::get_all_commands() {
        result.push(x);
    }
    for x in quests::get_all_commands() {
        result.push(x);
    }

    result
}

pub async fn send_error<'a>(ctx: &Context<'a>, content: &str) -> Result<(), Error> {
    send_ephemeral_reply(ctx, content).await?;
    Ok(())
}

pub async fn send_ephemeral_reply<'a>(
    ctx: &Context<'a>,
    content: &str,
) -> Result<ReplyHandle<'a>, serenity::Error> {
    ctx.send(|b| b.content(content).ephemeral(true)).await
}

#[allow(clippy::too_many_arguments)]
pub fn parse_variadic_args<T>(
    arg1: T,
    arg2: Option<T>,
    arg3: Option<T>,
    arg4: Option<T>,
    arg5: Option<T>,
    arg6: Option<T>,
    arg7: Option<T>,
    arg8: Option<T>,
    arg9: Option<T>,
) -> Vec<T> {
    let mut result = vec![arg1];
    add_if_some(&mut result, arg2);
    add_if_some(&mut result, arg3);
    add_if_some(&mut result, arg4);
    add_if_some(&mut result, arg5);
    add_if_some(&mut result, arg6);
    add_if_some(&mut result, arg7);
    add_if_some(&mut result, arg8);
    add_if_some(&mut result, arg9);

    result
}

fn add_if_some<T>(vec: &mut Vec<T>, option: Option<T>) {
    if let Some(x) = option {
        vec.push(x);
    }
}

pub async fn find_character(
    data: &Data,
    guild_id: u64,
    character_name: &str,
) -> Result<CharacterCacheItem, ParseError> {
    match parse_user_input_to_character(data, guild_id, character_name).await {
        Some(character) => Ok(character),
        None => Err(ParseError::new(&format!(
            "Unable to find a character named {}",
            character_name
        ))),
    }
}

pub async fn parse_user_input_to_character<'a>(
    data: &Data,
    guild_id: u64,
    text: &str,
) -> Option<CharacterCacheItem> {
    let characters = data.cache.get_characters().await;
    for x in &characters {
        if x.guild_id == guild_id && text == x.get_autocomplete_name() {
            return Some(x.clone());
        }
    }

    // User didn't use an autocomplete name :<
    let lowercase_input = text.to_lowercase();
    let name_matches: Vec<&CharacterCacheItem> = characters
        .iter()
        .filter(|x| x.guild_id == guild_id && x.name.to_lowercase() == lowercase_input)
        .collect();

    if name_matches.len() != 1 {
        None
    } else {
        name_matches.get(0).cloned().cloned()
    }
}

async fn parse_character_names<'a>(
    ctx: &Context<'a>,
    guild_id: u64,
    names: &Vec<String>,
) -> Result<Vec<CharacterCacheItem>, String> {
    let mut result: Vec<CharacterCacheItem> = Vec::new();

    for x in names {
        if let Some(character) = parse_user_input_to_character(ctx.data(), guild_id, x).await {
            result.push(character);
        } else {
            return Err(format!("Unable to find a character named {}", x));
        }
    }

    let mut ids = Vec::new();
    for x in &result {
        if ids.contains(&x.id) {
            return Err(format!(
                "Duplicate character: {}",
                x.get_autocomplete_name()
            ));
        }

        ids.push(x.id);
    }

    Ok(result)
}

pub async fn find_shop(
    data: &Data,
    guild_id: u64,
    shop_name: &str,
) -> Result<ShopCacheItem, ParseError> {
    match parse_user_input_to_shop(data, guild_id as i64, shop_name).await {
        Some(character) => Ok(character),
        None => Err(ParseError::new(&format!(
            "Unable to find a shop named {}",
            shop_name
        ))),
    }
}

pub async fn parse_user_input_to_shop<'a>(
    data: &Data,
    guild_id: i64,
    text: &str,
) -> Option<ShopCacheItem> {
    let entries = sqlx::query!(
        "SELECT name, id FROM shop WHERE shop.guild_id = ?",
        guild_id
    )
    .fetch_all(&data.database)
    .await;

    if let Ok(entries) = entries {
        let lowercase_input = text.to_lowercase();
        let name_matches: Vec<ShopCacheItem> = entries
            .iter()
            .filter(|x| x.name.to_lowercase() == lowercase_input)
            .map(|x| ShopCacheItem {
                id: x.id,
                name: x.name.clone(),
                guild_id: guild_id as u64,
            })
            .collect();

        if name_matches.len() != 1 {
            None
        } else {
            name_matches.get(0).cloned()
        }
    } else {
        None
    }
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

pub struct BuildUpdatedStatMessageStringResult {
    pub message: String,
    pub name: String,
    pub stat_channel_id: i64,
    pub stat_message_id: i64,
}

pub async fn ensure_character_has_money(
    ctx: &Context<'_>,
    character: &CharacterCacheItem,
    amount: i64,
    verb: &str,
) -> Result<(), ParseError> {
    let character_record = sqlx::query!("SELECT money FROM character WHERE id = ?", character.id)
        .fetch_one(&ctx.data().database)
        .await;

    if let Ok(character_record) = character_record {
        if character_record.money >= amount {
            Ok(())
        } else {
            Err(ParseError::new(&format!(
                "**Unable to {} {} {}.**\n*{} only owns {} {}.*",
                verb,
                amount,
                crate::emoji::POKE_COIN,
                character.name,
                character_record.money,
                crate::emoji::POKE_COIN
            )))
        }
    } else {
        Err(ParseError::new(format!("**Something went wrong when checking how much money {} has. Please try again. Let me know if this ever happens.**",
                                    character.name).as_str()
        ))
    }
}

pub fn ensure_user_owns_character(
    user: &User,
    giver: &CharacterCacheItem,
) -> Result<(), ParseError> {
    if giver.user_id == user.id.0 {
        Ok(())
    } else {
        Err(ParseError::new(&format!(
            "You don't seem to own a character named {} on this server.",
            giver.name
        )))
    }
}
