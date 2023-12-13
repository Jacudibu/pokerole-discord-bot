mod backups;
mod button_interaction;
mod quests;
mod role_reaction;
mod select_menu_interaction;

use crate::data::Data;
use crate::Error;
use poise::Event;
use serenity::client::Context;
use serenity::model::application::component::ComponentType;
use serenity::model::application::interaction::Interaction;
use serenity::model::id::ChannelId;
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use serenity::model::prelude::{GuildId, InteractionResponseType, User};

type FrameworkContext<'a> = poise::FrameworkContext<'a, Data, Error>;

pub async fn handle_events<'a>(
    context: &'a Context,
    event: &'a Event<'a>,
    framework: FrameworkContext<'a>,
) -> Result<(), Error> {
    match event {
        Event::InteractionCreate { interaction } => {
            handle_interaction(context, framework, interaction).await
        }
        Event::ReactionAdd { add_reaction } => {
            role_reaction::handle_reaction_add(context, framework, add_reaction).await
        }
        Event::ReactionRemove { removed_reaction } => {
            role_reaction::handle_reaction_remove(context, framework, removed_reaction).await
        }
        Event::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available: _,
        } => handle_guild_member_removal(context, framework.user_data, guild_id, user).await,
        Event::CacheReady { guilds: _ } => {
            backups::start_backup_thread(context, framework.user_data).await;
            Ok(())
        }
        _ => Ok(()),
    }
}

async fn handle_guild_member_removal(
    context: &Context,
    data: &Data,
    guild_id: &GuildId,
    user: &User,
) -> Result<(), Error> {
    // TODO: Should be a Database setting instead of being hardcoded.
    let channel_id: u64;
    let user_id = user.id.get() as i64;
    let guild_id = guild_id.get() as i64;
    if guild_id == 1113123066059436093 {
        // Explorers of the Sea
        channel_id = 1113127675586941140;
    } else if guild_id == 1115690620342763645 {
        // Test Server
        channel_id = 1120344272571486309;
    } else {
        return Ok(());
    }

    let character_names = sqlx::query!(
        "SELECT name FROM character WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id
    )
    .fetch_all(&data.database)
    .await;

    let names;
    if let Ok(character_names) = character_names {
        names = character_names
            .iter()
            .map(|x| x.name.clone())
            .collect::<Vec<String>>()
            .join(", ");
    } else {
        names = String::from("didn't find any characters for them in the database");
    }

    let channel = ChannelId::from(channel_id);
    channel
        .send_message(context, |create_message| {
            create_message
                .content(&format!("{} ({}) has left the server.", user, names))
                .allowed_mentions(|mentions| mentions.empty_users())
        })
        .await?;

    Ok(())
}

async fn handle_interaction(
    context: &Context,
    framework: FrameworkContext<'_>,
    interaction: &Interaction,
) -> Result<(), Error> {
    match interaction {
        Interaction::MessageComponent(component) => {
            handle_message_component_interaction(context, framework, component).await
        }
        _ => Ok(()),
    }
}

async fn handle_message_component_interaction(
    context: &Context,
    framework: FrameworkContext<'_>,
    interaction: &MessageComponentInteraction,
) -> Result<(), Error> {
    match interaction.data.component_type {
        ComponentType::ActionRow => {}
        ComponentType::Button => {
            button_interaction::handle_button_interaction(context, framework, &interaction).await?
        }
        ComponentType::SelectMenu => {
            select_menu_interaction::handle_select_menu_interaction(
                context,
                framework,
                &interaction,
            )
            .await?
        }
        ComponentType::InputText => {}
        ComponentType::Unknown => {}
        _ => {}
    }

    Ok(())
}

fn parse_interaction_command(custom_id: &str) -> (&str, Vec<&str>) {
    let mut split = custom_id.split('_');
    let command = split.next();
    let args: Vec<&str> = split.collect();

    (
        command.expect("Commands should never be empty at this point!"),
        args,
    )
}

async fn send_ephemeral_reply(
    interaction: &&MessageComponentInteraction,
    context: &Context,
    content: &str,
) -> Result<(), Error> {
    interaction
        .create_interaction_response(context, |interaction_response| {
            interaction_response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| data.ephemeral(true).content(content))
        })
        .await?;
    Ok(())
}

async fn send_error(
    interaction: &&MessageComponentInteraction,
    context: &Context,
    content: &str,
) -> Result<(), Error> {
    send_ephemeral_reply(interaction, context, content).await
}
