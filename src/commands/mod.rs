use std::borrow::Cow;

use poise::{Command, CreateReply, ReplyHandle};
use serenity::all::{ChannelId, CreateActionRow, EditMessage, Message, MessageId};
use serenity::model::guild::Member;
use serenity::model::id::{GuildId, UserId};
use serenity::model::prelude::User;

use crate::cache::{CharacterCacheItem, WalletCacheItem};
use crate::commands::characters::build_character_string;
use crate::data::Data;
use crate::errors::{ParseError, ValidationError};
use crate::game_data::pokemon::Pokemon;
use crate::{helpers, Error};

type Context<'a> = poise::Context<'a, Data, Error>;

mod autocompletion;

pub mod ability;
pub mod about;
pub mod calculate_hp_damage_modifier;
pub mod create_emojis;
mod create_role_reaction_post;
pub mod efficiency;
pub mod encounter;
pub mod item;
pub mod learns;
pub mod metronome;
pub mod r#move;
pub mod nature;
pub mod potion;
pub mod roll;
pub mod rule;
pub mod scale;
pub mod select_random;
pub mod stats;
pub mod status;
pub mod timestamp;
pub mod weather;

pub mod characters;
mod edit_rules;
mod pin_or_unpin;
mod player_info;
mod prune_emojis;
mod quests;
mod say;
mod server_stats;
mod setting_time_offset;
mod setup_emoji_guild;
mod setup_guild;
mod store_gm_experience;
mod use_gm_experience;
mod wallets;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    let mut result = vec![
        setup_guild::setup_guild(),
        setup_emoji_guild::setup_emoji_guild(),
        roll::roll(),
        roll::r(),
        r#move::poke_move(),
        ability::ability(),
        item::item(),
        stats::stats(),
        stats::pokemon(),
        status::status(),
        edit_rules::edit_rules(),
        rule::rule(),
        learns::learns(),
        nature::nature(),
        timestamp::timestamp(),
        weather::weather(),
        metronome::metronome(),
        efficiency::efficiency(),
        select_random::select_random(),
        player_info::player_info(),
        prune_emojis::prune_emojis(),
        scale::scale(),
        create_emojis::create_emojis(),
        encounter::encounter(),
        potion::potion(),
        calculate_hp_damage_modifier::calculate_hp_damage_modifier(),
        create_role_reaction_post::create_role_reaction_post(),
        setting_time_offset::setting_time_offset(),
        say::say(),
        about::about(),
        //prune_emojis::prune_emojis(),
        server_stats::server_stats(),
        pin_or_unpin::pin_or_unpin(),
        store_gm_experience::store_gm_experience(),
        use_gm_experience::use_gm_experience(),
    ];

    for x in characters::get_all_commands() {
        result.push(x);
    }
    for x in wallets::get_all_commands() {
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
    content: impl Into<String>,
) -> Result<ReplyHandle<'a>, serenity::Error> {
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await
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

pub async fn find_wallet(
    data: &Data,
    guild_id: u64,
    wallet_name: &str,
) -> Result<WalletCacheItem, ParseError> {
    match parse_user_input_to_wallet(data, guild_id as i64, wallet_name).await {
        Some(wallet) => Ok(wallet),
        None => Err(ParseError::new(&format!(
            "Unable to find a wallet named {}",
            wallet_name
        ))),
    }
}

pub async fn parse_user_input_to_wallet<'a>(
    data: &Data,
    guild_id: i64,
    text: &str,
) -> Option<WalletCacheItem> {
    let entries = sqlx::query!(
        "SELECT name, id FROM wallet WHERE wallet.guild_id = ?",
        guild_id
    )
    .fetch_all(&data.database)
    .await;

    if let Ok(entries) = entries {
        let lowercase_input = text.to_lowercase();
        let name_matches: Vec<WalletCacheItem> = entries
            .iter()
            .filter(|x| x.name.to_lowercase() == lowercase_input)
            .map(|x| WalletCacheItem {
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
    pub components: Vec<CreateActionRow>,
    pub name: String,
    pub stat_channel_id: i64,
    pub stat_message_id: i64,
}

#[derive(sqlx::FromRow)]
struct MoneyRecord {
    pub money: i64,
}

pub async fn ensure_character_has_money(
    data: &Data,
    character: &CharacterCacheItem,
    amount: i64,
    verb: &str,
) -> Result<(), ValidationError> {
    let character_record = sqlx::query_as("SELECT money FROM character WHERE id = ?")
        .bind(character.id)
        .fetch_one(&data.database)
        .await;

    ensure_money_record_has_money(&character.name, amount, verb, character_record)
}

pub async fn ensure_wallet_has_money(
    data: &Data,
    wallet: &WalletCacheItem,
    amount: i64,
    verb: &str,
) -> Result<(), ValidationError> {
    let record = sqlx::query_as("SELECT money FROM wallet WHERE id = ?")
        .bind(wallet.id)
        .fetch_one(&data.database)
        .await;

    ensure_money_record_has_money(&wallet.name, amount, verb, record)
}

fn ensure_money_record_has_money(
    entity_name: &str,
    amount: i64,
    verb: &str,
    character_record: Result<MoneyRecord, sqlx::Error>,
) -> Result<(), ValidationError> {
    if let Ok(character_record) = character_record {
        if character_record.money >= amount {
            Ok(())
        } else {
            Err(ValidationError::new(&format!(
                "**Unable to {} {} {}.**\n*{} only owns {} {}.*",
                verb,
                amount,
                crate::emoji::POKE_COIN,
                entity_name,
                character_record.money,
                crate::emoji::POKE_COIN
            )))
        }
    } else {
        Err(ValidationError::new(format!("**Something went wrong when checking how much money {} has. Please try again. Let me know if this ever happens.**",
                                         entity_name).as_str()
        ))
    }
}

pub fn ensure_user_owns_character(
    user: &User,
    character: &CharacterCacheItem,
) -> Result<(), ValidationError> {
    if character.user_id == user.id.get() {
        Ok(())
    } else {
        Err(ValidationError::new(&format!(
            "You don't seem to own a character named {} on this server.",
            character.name
        )))
    }
}

// TODO: Technically this should be persisted in the database and configurable on a per-server basis, but... as long as only one server uses the bot, who cares...? :D
const ADMIN_ROLE_ID: u64 = 1113123557292134480;
const GM_ROLE_ID: u64 = 1114261188323319878;
const TRIAL_GM_ROLE_ID: u64 = 1119538793310068787;

pub fn is_user_admin_or_gm(user_member: Cow<'_, Member>) -> bool {
    user_member
        .roles
        .iter()
        .any(|r| r.get() == ADMIN_ROLE_ID || r.get() == GM_ROLE_ID || r.get() == TRIAL_GM_ROLE_ID)
}

pub async fn ensure_user_owns_wallet_or_is_gm(
    data: &Data,
    user_id: i64,
    user_member: Cow<'_, Member>,
    wallet: &WalletCacheItem,
) -> Result<(), ValidationError> {
    if is_user_admin_or_gm(user_member) {
        return Ok(());
    }

    let owners = sqlx::query!(
        "SELECT * FROM character WHERE user_id = ? and id in (\
                SELECT character_id FROM wallet_owner WHERE wallet_id = ?)",
        user_id,
        wallet.id
    )
    .fetch_all(&data.database)
    .await;

    if let Ok(owners) = owners {
        if owners.is_empty() {
            Err(ValidationError::new(
                "Only wallet owners, GMs and Admins can withdraw money from wallet wallets.",
            ))
        } else {
            Ok(())
        }
    } else {
        Err(ValidationError::new("Was unable to validate whether you are allowed to access this wallet. Please try again."))
    }
}

async fn update_character_post<'a>(ctx: &Context<'a>, id: i64) {
    if let Some(result) = build_character_string(&ctx.data().database, &ctx.data().game, id).await {
        let message = ctx
            .serenity_context()
            .http
            .get_message(
                ChannelId::from(result.stat_channel_id as u64),
                MessageId::from(result.stat_message_id as u64),
            )
            .await;
        if let Ok(mut message) = message {
            if let Err(e) = message
                .edit(
                    ctx,
                    EditMessage::new()
                        .content(&result.message)
                        .components(result.components.clone()),
                )
                .await
            {
                handle_error_during_message_edit(
                    ctx,
                    e,
                    message,
                    result.message,
                    Some(result.components),
                    result.name,
                )
                .await;
            }
        }
    }
}

async fn handle_error_during_message_edit<'a>(
    ctx: &Context<'a>,
    e: serenity::Error,
    message_to_edit: Message,
    updated_message_content: impl Into<String>,
    components: Option<Vec<CreateActionRow>>,
    name: impl Into<String>,
) {
    helpers::handle_error_during_message_edit(
        ctx.serenity_context(),
        e,
        message_to_edit,
        updated_message_content,
        components,
        name,
        Some(ctx.channel_id()),
    )
    .await;
}

fn pokemon_from_autocomplete_string<'a>(
    ctx: &Context<'a>,
    name: &String,
) -> Result<&'a Pokemon, ParseError> {
    let pokemon = ctx.data().game.pokemon.get(&name.to_lowercase());
    if let Some(pokemon) = pokemon {
        Ok(pokemon)
    } else {
        Err(ParseError::new(&std::format!("Unable to find a pokemon named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name)))
    }
}

pub struct ServerData {
    id: i64,
    name: Option<String>,
}

async fn get_servers_this_user_is_active_in(
    ctx: &Context<'_>,
) -> Result<Vec<ServerData>, sqlx::Error> {
    let user_id = ctx.author().id.get() as i64;

    sqlx::query_as!(
        ServerData,
        "
SELECT guild.id as id, guild.name as name
FROM guild
JOIN user_in_guild
    ON guild.id = user_in_guild.guild_id
WHERE user_in_guild.user_id = ? and guild.name IS NOT NULL",
        user_id
    )
    .fetch_all(&ctx.data().database)
    .await
}
