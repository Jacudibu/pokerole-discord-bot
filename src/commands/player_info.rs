use serenity::all::{ChannelId, Member, User};
use tokio::join;

use crate::commands::Error;
use crate::shared::data::Data;
use crate::shared::errors::DatabaseError;
use crate::shared::game_data::PokemonApiId;
use crate::shared::utility::channel_id_ext::ChannelIdExt;
use crate::shared::utility::message_splitting::split_long_messages;
use crate::shared::utility::{error_handling, level_calculations};
use crate::shared::{emoji, PoiseContext};

/// Display Stats for a player
#[poise::command(slash_command, guild_only)]
pub async fn player_info(
    ctx: PoiseContext<'_>,
    #[description = "Which player?"] player: User,
) -> Result<(), Error> {
    let user_id = player.id.get() as i64;
    let guild = ctx.guild().expect("Command is guild_only").clone();
    let guild_id = guild.id.get() as i64;

    let user_in_guild = guild.member(&ctx, player.id);
    let characters = sqlx::query_as!(QueryObject, "SELECT name, species_api_id, experience, stat_channel_id FROM character WHERE user_id = ? AND guild_id = ? AND is_retired = false", user_id, guild_id)
        .fetch_all(&ctx.data().database);
    let hosted_quest_count = query_hosted_quest_count(&ctx, user_id, guild_id);
    let gm_experience = query_gm_experience(&ctx, user_id, guild_id);

    let (user_in_guild, characters, hosted_quest_count, gm_experience) =
        join!(user_in_guild, characters, hosted_quest_count, gm_experience);

    let user_in_guild = user_in_guild.expect("Player must be part of this server!");

    match characters {
        Ok(characters) => {
            let reply = build_reply(
                ctx.serenity_context(),
                ctx.data(),
                &user_in_guild,
                characters,
                hosted_quest_count,
                gm_experience,
            )
            .await;
            for message in split_long_messages(reply) {
                let _ = ctx.reply(message).await;
            }
        }
        Err(e) => {
            return Err(Box::new(DatabaseError::new(&format!(
                "Encountered an error when searching for characters for user {}: {}",
                user_in_guild, e
            ))));
        }
    }

    Ok(())
}

async fn query_hosted_quest_count(
    ctx: &PoiseContext<'_>,
    user_id: i64,
    guild_id: i64,
) -> Option<i64> {
    match sqlx::query!(
        "SELECT COUNT(*) as count FROM quest WHERE creator_id = ? AND guild_id = ? AND completion_timestamp IS NOT NULL",
        user_id,
        guild_id
    )
        .fetch_one(&ctx.data().database)
        .await {
        Ok(record) => Some(record.count),
        Err(_) => None
    }
}

async fn query_gm_experience(ctx: &PoiseContext<'_>, user_id: i64, guild_id: i64) -> Option<i64> {
    match sqlx::query!(
        "SELECT gm_experience FROM user_in_guild WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id
    )
    .fetch_one(&ctx.data().database)
    .await
    {
        Ok(record) => Some(record.gm_experience),
        Err(_) => None,
    }
}

async fn build_reply(
    ctx: &serenity::all::Context,
    data: &Data,
    user_in_guild: &Member,
    characters: Vec<QueryObject>,
    hosted_quest_count: Option<i64>,
    gm_experience: Option<i64>,
) -> String {
    let mut character_list = String::new();
    let character_count = characters.len();
    let mut total_levels = 0;
    let mut total_exp = 0;
    for character in characters {
        total_exp += character.experience;

        let character_level =
            level_calculations::calculate_level_from_experience(character.experience);
        total_levels += character_level;
        let current_exp = level_calculations::calculate_current_experience(character.experience);

        let channel_id = ChannelId::new(character.stat_channel_id as u64);
        let api_id = PokemonApiId(character.species_api_id as u16);

        let Some(pokemon) = data.game.base_data.pokemon_by_api_id.get(&api_id) else {
            error_handling::log_error(ctx, format!(
                "Database values should always be valid, but couldn't find an API entry for character with id {:?} and poke_api id {:?}",
                user_in_guild.user.id,
                api_id
            )).await;

            continue;
        };
        let emoji = emoji::get_any_pokemon_emoji_with_space(ctx, &data.database, pokemon);

        character_list.push_str(&format!(
            "### {}{} – {}\n        Level: {} ({} exp)\n",
            emoji.await,
            character.name,
            channel_id.channel_id_link(),
            character_level,
            current_exp
        ))
    }

    let joined = if let Some(date) = user_in_guild.joined_at {
        date.format("%c").to_string()
    } else {
        String::from("Unknown")
    };

    let hosted_quest_count = if let Some(hosted_quest_count) = hosted_quest_count {
        format!("\n**Hosted Quests:** {}", hosted_quest_count)
    } else {
        String::new()
    };

    let gm_experience = if let Some(gm_experience) = gm_experience {
        format!("\n**GM Experience**: {}", gm_experience)
    } else {
        String::new()
    };

    let character_slots = 1 + total_levels / 5;

    format!(
        "## {}
**Joined at**: {}
**Total Character Level**: {} 
**Total Experience**: {}
**Character Slots**: {}/{}{}{}
{}",
        user_in_guild.display_name(),
        joined,
        total_levels,
        total_exp,
        character_count,
        character_slots,
        hosted_quest_count,
        gm_experience,
        character_list
    )
}

struct QueryObject {
    name: String,
    species_api_id: i64,
    experience: i64,
    stat_channel_id: i64,
}
