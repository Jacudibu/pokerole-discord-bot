use crate::Error;
use crate::events::FrameworkContext;
use crate::shared::emoji;
use log::info;
use serenity::all::MessageInteractionMetadata;
use serenity::client::Context;
use serenity::model::channel::{Reaction, ReactionType};

pub async fn handle_reaction_add(
    ctx: &Context,
    _framework: FrameworkContext<'_>,
    reaction: &Reaction,
) -> Result<(), Error> {
    let emoji_name = get_emoji_name(&reaction.emoji);
    match emoji_name.as_str() {
        emoji::UNICODE_CROSS_MARK | emoji::UNICODE_CROSS_MARK_BUTTON => {
            delete_bot_message(ctx, reaction).await
        }
        _ => Ok(()),
    }
}

async fn delete_bot_message(ctx: &Context, reaction: &Reaction) -> Result<(), Error> {
    if let Some(user_id) = reaction.user_id {
        let message = reaction.message(ctx).await?;
        if message.author.bot && ctx.cache.current_user().id == message.author.id {
            if let Some(interaction) = message.interaction_metadata {
                let interaction_user_id = match *interaction {
                    MessageInteractionMetadata::Command(data) => Some(data.user.id),
                    _ => None,
                };

                if let Some(interaction_user_id) = interaction_user_id {
                    if interaction_user_id == user_id {
                        ctx.http
                            .delete_message(
                                reaction.channel_id,
                                reaction.message_id,
                                Some("Delete emoji was sent."),
                            )
                            .await?;
                    }
                } else {
                    info!(
                        "Encountered invalid message interaction metadata when trying to delete a message!"
                    )
                }
            }
        }
    }
    Ok(())
}

pub async fn handle_reaction_remove(
    _ctx: &Context,
    _framework: FrameworkContext<'_>,
    _reaction: &Reaction,
) -> Result<(), Error> {
    Ok(())
}

fn get_emoji_name(reaction: &ReactionType) -> String {
    match reaction {
        ReactionType::Custom {
            animated: _animated,
            id: _id,
            name,
        } => name.clone().unwrap_or(String::new()),
        ReactionType::Unicode(value) => value.clone(),
        _ => String::new(),
    }
}
