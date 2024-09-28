use crate::data::Data;
use crate::game_data::GameData;
use crate::{helpers, Error};
use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, CreateActionRow, CreateAllowedMentions,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EditMessage,
    FullEvent, GuildId, Interaction, Member, Message, MessageId, RoleId, User,
};
use serenity::client::Context;
use serenity::model::id::ChannelId;
use sqlx::{Pool, Sqlite};
use tokio::join;

mod backups;
mod button_interaction;
mod character_stat_edit;
mod monthly_reset;
mod quests;
mod role_reaction;
mod select_menu_interaction;
mod status_messages;
mod weekly_reset;

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
        FullEvent::GuildMemberUpdate {
            old_if_available,
            new,
            event,
        } => {
            if let Some(new) = new {
                handle_guild_member_update(context, &framework.user_data, new).await
            } else {
                let _ = helpers::ERROR_LOG_CHANNEL
                    .send_message(&context, CreateMessage::new().content(
                        format!("Encountered a weird edge case in GuildMemberUpdate.\n old: {:?}\n new: {:?}, event: {:?}",
                                old_if_available, new, event)))
                    .await;

                Ok(())
            }
        }
        FullEvent::GuildMemberAddition { new_member } => {
            handle_guild_member_addition(context, &framework.user_data, new_member).await
        }
        FullEvent::MessageDelete {
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            // TODO: Maybe log message deletion
            Ok(())
        }
        FullEvent::MessageUpdate {
            old_if_available,
            new,
            event,
        } => {
            // TODO: Maybe log message edit
            Ok(())
        }
        FullEvent::Ready { .. } => {
            join!(
                backups::start_backup_thread(context, framework.user_data),
                weekly_reset::start_weekly_reset_thread(context, framework.user_data),
                monthly_reset::start_monthly_reset_thread(context, framework.user_data),
                status_messages::restart_message(context, framework.user_data),
            );
            Ok(())
        }
        _ => Ok(()),
    }
}

async fn handle_guild_member_addition(
    ctx: &Context,
    data: &Data,
    new_member: &Member,
) -> Result<(), Error> {
    let guild_id = new_member.guild_id.get() as i64;

    match sqlx::query!(
        "SELECT default_member_role_id FROM guild WHERE id = ?",
        guild_id
    )
    .fetch_optional(&data.database)
    .await
    {
        Ok(record) => {
            if let Some(record) = record {
                if let Some(default_member_role_id) = record.default_member_role_id {
                    let role = RoleId::new(default_member_role_id as u64);
                    if let Err(e) = new_member.add_role(ctx, role).await {
                        send_error_to_log_channel(
                            ctx,
                            format!("Failed setting default role for new user: {e}"),
                        )
                        .await;
                    }
                }
            }
        }
        Err(_) => {
            // database ded?
        }
    }
    handle_guild_member_update(ctx, data, new_member).await?;
    Ok(())
}

async fn handle_guild_member_update(ctx: &Context, data: &Data, new: &Member) -> Result<(), Error> {
    let user_id = new.user.id.get() as i64;
    let guild_id = new.guild_id.get() as i64;

    let nickname = match &new.nick {
        None => &new.user.name,
        Some(nick) => nick,
    };

    let result = sqlx::query!(
        "
INSERT INTO user_in_guild (name, user_id, guild_id) VALUES (?, ?, ?) 
ON CONFLICT(user_id, guild_id) DO UPDATE SET name = ?",
        nickname,
        user_id,
        guild_id,
        nickname,
    )
    .execute(&data.database)
    .await;

    data.cache.reset(&data.database).await;

    Ok(())
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

async fn update_character_post<'a>(
    ctx: &Context,
    database: &Pool<Sqlite>,
    game_data: &GameData,
    id: i64,
) {
    if let Some(result) =
        crate::commands::characters::build_character_string(database, game_data, id).await
    {
        let message = ctx
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

async fn handle_error_during_message_edit(
    ctx: &Context,
    e: serenity::Error,
    message_to_edit: Message,
    updated_message_content: impl Into<String>,
    components: Option<Vec<CreateActionRow>>,
    name: impl Into<String>,
) {
    helpers::handle_error_during_message_edit(
        ctx,
        e,
        message_to_edit,
        updated_message_content,
        components,
        name,
        None,
    )
    .await;
}
