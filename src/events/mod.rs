mod backups;
mod button_interaction;
mod quests;
mod role_reaction;
mod select_menu_interaction;
mod weekly_reset;

use crate::data::Data;
use crate::{helpers, Error};
use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, CreateAllowedMentions,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, FullEvent, GuildId,
    Interaction, User,
};
use serenity::client::Context;
use serenity::model::id::ChannelId;

type FrameworkContext<'a> = poise::FrameworkContext<'a, Data, Error>;

pub async fn handle_events<'a>(
    context: &'a Context,
    event: &FullEvent,
    framework: FrameworkContext<'a>,
) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate { interaction } => {
            handle_interaction(context, framework, interaction).await
        }
        FullEvent::ReactionAdd { add_reaction } => {
            role_reaction::handle_reaction_add(context, framework, add_reaction).await
        }
        FullEvent::ReactionRemove { removed_reaction } => {
            role_reaction::handle_reaction_remove(context, framework, removed_reaction).await
        }
        FullEvent::GuildMemberRemoval { guild_id, user, .. } => {
            handle_guild_member_removal(context, framework.user_data, guild_id, user).await
        }
        FullEvent::Ready { .. } => {
            // TODO: Could use the data inside this event to lazily count how many discord servers are using the bot.
            backups::start_backup_thread(context, framework.user_data).await;
            weekly_reset::start_weekly_reset_thread(context, framework.user_data).await;
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
    let user_name = &user.name;
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
        if character_names.is_empty() {
            names = String::from("didn't find any characters for them in the database");
        } else {
            names = character_names
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>()
                .join(", ");
        }
    } else {
        names = String::from("failed to check database for matching character names...?");
    }

    let channel = ChannelId::from(channel_id);
    channel
        .send_message(
            context,
            CreateMessage::new()
                .content(&format!(
                    "{}/{} ({}) has left the server.",
                    user_name, user, names
                ))
                .allowed_mentions(CreateAllowedMentions::default().empty_users()),
        )
        .await?;

    Ok(())
}

async fn handle_interaction(
    context: &Context,
    framework: FrameworkContext<'_>,
    interaction: &Interaction,
) -> Result<(), Error> {
    match interaction {
        Interaction::Component(component) => {
            handle_message_component_interaction(context, framework, component).await
        }
        _ => Ok(()),
    }
}

async fn handle_message_component_interaction(
    context: &Context,
    framework: FrameworkContext<'_>,
    interaction: &ComponentInteraction,
) -> Result<(), Error> {
    match &interaction.data.kind {
        ComponentInteractionDataKind::Button => {
            button_interaction::handle_button_interaction(context, framework, &interaction).await?
        }
        ComponentInteractionDataKind::StringSelect { .. } => {
            select_menu_interaction::handle_select_menu_interaction(
                context,
                framework,
                &interaction,
            )
            .await?
        }
        ComponentInteractionDataKind::UserSelect { .. } => {}
        ComponentInteractionDataKind::RoleSelect { .. } => {}
        ComponentInteractionDataKind::MentionableSelect { .. } => {}
        ComponentInteractionDataKind::ChannelSelect { .. } => {}
        ComponentInteractionDataKind::Unknown(_) => {}
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
    interaction: &&ComponentInteraction,
    context: &Context,
    content: &str,
) -> Result<(), Error> {
    interaction
        .create_response(
            context,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content(content),
            ),
        )
        .await?;
    Ok(())
}

async fn send_error(
    interaction: &&ComponentInteraction,
    context: &Context,
    content: &str,
) -> Result<(), Error> {
    send_ephemeral_reply(interaction, context, content).await
}

async fn send_error_to_log_channel(ctx: &Context, message: impl Into<String>) {
    let _ = helpers::ERROR_LOG_CHANNEL
        .send_message(ctx, CreateMessage::new().content(message))
        .await;
}
