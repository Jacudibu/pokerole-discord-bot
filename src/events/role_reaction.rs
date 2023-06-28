use crate::events::FrameworkContext;
use crate::{emoji, Error};
use serenity::client::Context;
use serenity::model::channel::{Reaction, ReactionType};
use serenity::model::prelude::interaction::InteractionType;

// TODO: Move this into a database.
const ALLOWED_MESSAGE_IDS: [u64; 3] = [
    1119664626188156978,
    1119665389899616396,
    1119666186867716137,
];

pub async fn handle_reaction_add(
    ctx: &Context,
    _framework: FrameworkContext<'_>,
    reaction: &Reaction,
) -> Result<(), Error> {
    let emoji_name = get_emoji_name(&reaction.emoji);
    if ALLOWED_MESSAGE_IDS.contains(&reaction.message_id.0) {
        let role_id = emoji_to_role_id(emoji_name.as_str());
        ctx.http
            .add_member_role(
                reaction.guild_id.unwrap().0,
                reaction.user_id.unwrap().0,
                role_id,
                Some("Clicked the button to add the role."),
            )
            .await?;

        return Ok(());
    }

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
        if message.author.bot && ctx.cache.current_user_id() == message.author.id {
            if let Some(interaction) = message.interaction {
                if interaction.kind == InteractionType::ApplicationCommand
                    && interaction.user.id == user_id
                {
                    ctx.http
                        .delete_message(reaction.channel_id.0, reaction.message_id.0)
                        .await?;
                }
            }
        }
    }
    Ok(())
}

pub async fn handle_reaction_remove(
    ctx: &Context,
    _framework: FrameworkContext<'_>,
    reaction: &Reaction,
) -> Result<(), Error> {
    if ALLOWED_MESSAGE_IDS.contains(&reaction.message_id.0) {
        let bla = get_emoji_name(&reaction.emoji);
        let role_id = emoji_to_role_id(bla.as_str());
        ctx.http
            .remove_member_role(
                reaction.guild_id.unwrap().0,
                reaction.user_id.unwrap().0,
                role_id,
                Some("Clicked the button to remove the role."),
            )
            .await?;
    }

    Ok(())
}

fn emoji_to_role_id(emoji_name: &str) -> u64 {
    match emoji_name {
        "❤️" => 1115475058958278707,
        "💙" => 1115475277611544596,
        "💛" => 1115475324264792074,
        "🍆" => 1115475361380188171,
        "🐱" => 1115475400215244861,
        "🔔" => 1115475494956179598,
        "⚙️" => 1115475607950721165,
        "🤖" => 1116615590308749352,
        "💬" => 1115475646668353577,
        "📣" => 1119659198968512655,
        _ => panic!("unexpected emoji name! {}", emoji_name),
    }
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
