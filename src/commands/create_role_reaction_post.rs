use crate::commands::Error;
use crate::shared::PoiseContext;
use poise::CreateReply;
use serenity::all::CreateMessage;
use serenity::model::channel::Channel;
use serenity::model::channel::ReactionType;
use serenity::model::guild::Role;
use serenity::utils::MessageBuilder;

/// WIP. Create a post for role reactions.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
#[allow(clippy::too_many_arguments)]
pub async fn create_role_reaction_post(
    ctx: PoiseContext<'_>,
    #[description = "In which channel?"] channel: Channel,
    #[description = "What should be the post title?"] text: String,
    #[description = "The emoji for the first role"] emoji_1: String,
    #[description = "The role for the first role"] role_1: Role,
    emoji_2: String,
    role_2: Role,
    emoji_3: Option<String>,
    role_3: Option<Role>,
    emoji_4: Option<String>,
    role_4: Option<Role>,
    emoji_5: Option<String>,
    role_5: Option<Role>,
    emoji_6: Option<String>,
    role_6: Option<Role>,
    emoji_7: Option<String>,
    role_7: Option<Role>,
    emoji_8: Option<String>,
    role_8: Option<Role>,
    emoji_9: Option<String>,
    role_9: Option<Role>,
) -> Result<(), Error> {
    if ctx.author().id.get() != 878982444412448829 {
        ctx.send(CreateReply::default()
            .content("This command is currently highly WIP and requires some manual hacks to work. Sowwie! Contact Lilo if you really need to use it.")
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    let mut reaction_message = MessageBuilder::default();
    let mut command_response = MessageBuilder::default();
    let mut reactions = Vec::default();
    reaction_message.push_bold_line(text);

    add_role(
        &mut reaction_message,
        &mut command_response,
        emoji_1,
        role_1,
        &mut reactions,
    );
    add_role(
        &mut reaction_message,
        &mut command_response,
        emoji_2,
        role_2,
        &mut reactions,
    );
    add_optional_role(
        &mut reaction_message,
        &mut command_response,
        emoji_3,
        role_3,
        &mut reactions,
    );
    add_optional_role(
        &mut reaction_message,
        &mut command_response,
        emoji_4,
        role_4,
        &mut reactions,
    );
    add_optional_role(
        &mut reaction_message,
        &mut command_response,
        emoji_5,
        role_5,
        &mut reactions,
    );
    add_optional_role(
        &mut reaction_message,
        &mut command_response,
        emoji_6,
        role_6,
        &mut reactions,
    );
    add_optional_role(
        &mut reaction_message,
        &mut command_response,
        emoji_7,
        role_7,
        &mut reactions,
    );
    add_optional_role(
        &mut reaction_message,
        &mut command_response,
        emoji_8,
        role_8,
        &mut reactions,
    );
    add_optional_role(
        &mut reaction_message,
        &mut command_response,
        emoji_9,
        role_9,
        &mut reactions,
    );

    let message = channel
        .id()
        .send_message(ctx, CreateMessage::new().content(reaction_message.build()))
        .await?;

    for x in reactions {
        message.react(ctx, ReactionType::Unicode(x)).await?;
    }

    ctx.say(format!(
        "Message created.\nMessageId: `{}`\n{}",
        message.id, command_response
    ))
    .await?;

    Ok(())
}

fn add_optional_role(
    reaction_message: &mut MessageBuilder,
    command_response: &mut MessageBuilder,
    emoji: Option<String>,
    role: Option<Role>,
    all_emojis: &mut Vec<String>,
) {
    if let Some(emoji) = emoji {
        add_role(
            reaction_message,
            command_response,
            emoji,
            role.unwrap(),
            all_emojis,
        );
    }
}

fn add_role(
    reaction_message: &mut MessageBuilder,
    command_response: &mut MessageBuilder,
    emoji: String,
    role: Role,
    all_emojis: &mut Vec<String>,
) {
    command_response.push(&emoji);
    command_response.push_mono(role.id.get().to_string());
    command_response.push(' ');
    command_response.push_mono_line("@".to_owned() + &role.name);

    reaction_message.push(&emoji);
    reaction_message.push(' ');
    reaction_message.push_line(role.name);

    all_emojis.push(emoji);
}
