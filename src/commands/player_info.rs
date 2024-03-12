use serenity::all::{ChannelId, Member, User};

use crate::{emoji, helpers};
use crate::commands::{Context, Error};
use crate::data::Data;
use crate::errors::DatabaseError;
use crate::game_data::PokemonApiId;
use crate::helpers::split_long_messages;

/// Display Stats for a player
#[poise::command(slash_command, guild_only)]
pub async fn player_info(
    ctx: Context<'_>,
    #[description = "Which player?"] player: User,
) -> Result<(), Error> {
    let user_id = player.id.get() as i64;
    let guild = ctx.guild().expect("Command is guild_only").clone();
    let guild_id = guild.id.get() as i64;
    let user_in_guild = guild
        .member(&ctx, player.id)
        .await
        .expect("Player must be part of this server!");

    let characters = sqlx::query_as!(QueryObject, "SELECT id, name, species_api_id, experience, is_shiny, phenotype, stat_channel_id FROM character WHERE user_id = ? AND guild_id = ? AND is_retired = false", user_id, guild_id)
        .fetch_all(&ctx.data().database)
        .await;

    let hosted_quest_count = match sqlx::query!(
        "SELECT COUNT(*) as count FROM quest WHERE creator_id = ? AND completion_timestamp IS NOT NULL",
        user_id,
    )
    .fetch_one(&ctx.data().database)
    .await {
        Ok(record) => {record.count}
        Err(_) => {0}
    };

    let gm_experience = match sqlx::query!(
        "SELECT gm_experience FROM user_in_guild WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id
    )
    .fetch_one(&ctx.data().database)
    .await
    {
        Ok(record) => Some(record.gm_experience),
        Err(_) => None,
    };

    match characters {
        Ok(characters) => {
            let reply = build_reply(
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

async fn build_reply(
    data: &Data,
    user_in_guild: &Member,
    characters: Vec<QueryObject>,
    hosted_quest_count: i32,
    gm_experience: Option<i64>,
) -> String {
    let mut character_list = String::new();
    let character_count = characters.len();
    let mut total_levels = 0;
    let mut total_exp = 0;
    for character in characters {
        total_exp += character.experience;

        let character_level = helpers::calculate_level_from_experience(character.experience);
        total_levels += character_level;
        let current_exp = helpers::calculate_current_experience(character.experience);

        let channel_id = ChannelId::new(character.stat_channel_id as u64);
        let api_id = PokemonApiId(character.species_api_id as u16);

        let pokemon = data
            .game
            .pokemon_by_api_id
            .get(&api_id)
            .expect("Database values should always be valid!");
        let emoji = emoji::get_any_pokemon_emoji_with_space(&data.database, pokemon);

        character_list.push_str(&format!(
            "### {}{} – {}
        Level: {} ({} exp)\n",
            emoji.await,
            character.name,
            helpers::channel_id_link(channel_id),
            character_level,
            current_exp
        ))
    }

    let joined = if let Some(date) = user_in_guild.joined_at {
        date.format("%c").to_string()
    } else {
        String::from("Unknown")
    };

    let hosted_quest_count = if hosted_quest_count > 0 {
        format!("\n**Hosted Quests:** {}", hosted_quest_count)
    } else {
        String::new()
    };

    let gm_experience = if let Some(gm_experience) = gm_experience {
        format!("\nGM Experience: {}", gm_experience)
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
    id: i64,
    name: String,
    species_api_id: i64,
    experience: i64,
    phenotype: i64,
    is_shiny: bool,
    stat_channel_id: i64,
}
