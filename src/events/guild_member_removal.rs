use crate::Error;
use crate::shared::SerenityContext;
use crate::shared::action_log::LogActionArguments;
use crate::shared::data::Data;
use crate::shared::retire_character::retire_character_with_id;
use serenity::all::{ChannelId, CreateAllowedMentions, CreateMessage, GuildId, User};

struct QueryItem {
    id: i64,
    name: String,
}

pub async fn handle_guild_member_removal(
    context: &SerenityContext,
    data: &Data,
    guild_id: &GuildId,
    user: &User,
) -> Result<(), Error> {
    let user_id = user.id.get() as i64;
    let guild_id = guild_id.get() as i64;

    let characters = sqlx::query_as!(
        QueryItem,
        "SELECT name, id FROM character WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id
    )
    .fetch_all(&data.database)
    .await;

    if let Ok(characters) = &characters {
        for character in characters {
            let _ = retire_character_with_id(
                context,
                data,
                LogActionArguments::triggered_by_bot(context, &data.database),
                character.id,
                user.id,
                &character.name,
            )
            .await;
        }
    }

    notify_server(context, user, guild_id, characters).await
}

async fn notify_server(
    context: &SerenityContext,
    user: &User,
    guild_id: i64,
    characters: Result<Vec<QueryItem>, sqlx::Error>,
) -> Result<(), Error> {
    // TODO: This should be a configurable Database setting instead of being hardcoded.

    let channel_id = if guild_id == 1113123066059436093 {
        // Explorers of the Sea
        Some(1113127675586941140)
    } else if guild_id == 1115690620342763645 {
        // Test Server
        Some(1120344272571486309)
    } else {
        None
    };

    let Some(channel_id) = channel_id else {
        return Ok(());
    };

    let names = if let Ok(characters) = &characters {
        if characters.is_empty() {
            String::from("didn't find any characters for them in the database")
        } else {
            characters
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>()
                .join(", ")
        }
    } else {
        String::from("failed to check database for matching character names...?")
    };

    let channel = ChannelId::from(channel_id);
    channel
        .send_message(
            context,
            CreateMessage::new()
                .content(format!(
                    "{}/{} ({}) has left the server.",
                    user.name, user, names
                ))
                .allowed_mentions(CreateAllowedMentions::default().empty_users()),
        )
        .await?;

    Ok(())
}
