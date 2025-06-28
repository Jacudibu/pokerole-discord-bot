use std::str::FromStr;

use serenity::all::{
    ActionRow, ActionRowComponent, Button, ButtonKind, ComponentInteraction,
    CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};
use serenity::builder::{CreateActionRow, CreateButton};
use serenity::client::Context;

use crate::events::{
    FrameworkContext, character_stat_edit, parse_interaction_command, quests, send_ephemeral_reply,
};
use crate::shared::errors::CommandInvocationError;
use crate::shared::game_data::GameData;
use crate::shared::utility::channel_id_ext::ChannelIdExt;
use crate::shared::utility::message_splitting;
use crate::shared::{clunky_stuff, dice_rolls, emoji};
use crate::{Error, shared};

async fn get_game_data<'a>(
    framework: &'a FrameworkContext<'_>,
    interaction: &&ComponentInteraction,
) -> &'a GameData {
    framework
        .user_data
        .game
        .get_by_interaction(interaction, &framework.user_data.database)
        .await
}

pub async fn handle_button_interaction(
    context: &Context,
    framework: FrameworkContext<'_>,
    interaction: &&ComponentInteraction,
) -> Result<(), Error> {
    if interaction.data.custom_id.is_empty() {
        return Ok(());
    }

    let (command, args) = parse_interaction_command(interaction.data.custom_id.as_str());
    match command {
        "metronome" => {
            disable_button_on_original_message(context, interaction).await?;
            let game_data = get_game_data(&framework, interaction).await;

            interaction
                .message
                .reply(context, shared::metronome::get_metronome_text(game_data))
                .await?;
        }
        "ignore" => {
            interaction
                .create_response(
                    context,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content("That tickles!"),
                    ),
                )
                .await?;
        }
        "learns-all" => {
            disable_button_on_original_message(context, interaction).await?;
            let game_data = get_game_data(&framework, interaction).await;
            let pokemon = game_data.pokemon.get(args[0]).unwrap();
            let emoji = emoji::get_any_pokemon_emoji_with_space(
                context,
                &framework.user_data.database,
                pokemon,
            )
            .await;
            for response_part in message_splitting::split_long_messages(
                pokemon.build_all_learnable_moves_list(emoji).into(),
            ) {
                interaction.message.reply(context, response_part).await?;
            }
        }
        "efficiency" => {
            disable_button_on_original_message(context, interaction).await?;
            let game_data = get_game_data(&framework, interaction).await;
            let pokemon = game_data.pokemon.get(args[0]).unwrap();
            let emoji = emoji::get_any_pokemon_emoji_with_space(
                context,
                &framework.user_data.database,
                pokemon,
            )
            .await;
            interaction
                .message
                .reply(
                    context,
                    &framework
                        .user_data
                        .game
                        .type_efficiency
                        .get_type_resistances_string(pokemon, emoji),
                )
                .await?;
        }
        "pokedex" => {
            disable_button_on_original_message(context, interaction).await?;
            let game_data = get_game_data(&framework, interaction).await;
            let pokemon = game_data.pokemon.get(args[0]).unwrap();
            for response_part in
                message_splitting::split_long_messages(pokemon.build_pokedex_string())
            {
                interaction.message.reply(context, response_part).await?;
            }
        }
        "moves" => {
            disable_button_on_original_message(context, interaction).await?;
            let game_data = get_game_data(&framework, interaction).await;
            let pokemon = game_data.pokemon.get(args[0]).unwrap();
            let emoji = emoji::get_any_pokemon_emoji_with_space(
                context,
                &framework.user_data.database,
                pokemon,
            )
            .await;
            interaction
                .create_followup(
                    context,
                    clunky_stuff::create_learns_reply(pokemon, emoji)
                        .to_slash_followup_response(CreateInteractionResponseFollowup::new()),
                )
                .await?;
        }
        "abilities" => {
            disable_button_on_original_message(context, interaction).await?;
            let game_data = get_game_data(&framework, interaction).await;
            let pokemon = game_data.pokemon.get(args[0]).unwrap();
            let emoji = emoji::get_any_pokemon_emoji_with_space(
                context,
                &framework.user_data.database,
                pokemon,
            )
            .await;
            interaction
                .message
                .reply(
                    context,
                    pokemon
                        .build_ability_string(emoji, &game_data.abilities)
                        .into(),
                )
                .await?;
        }
        "roll-dice" => {
            let message = dice_rolls::parse_query(args[0])
                .expect("This should always be a valid query in buttons!")
                .execute();
            interaction
                .create_response(
                    context,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content(message),
                    ),
                )
                .await?;
        }
        "quest-sign-up" => {
            quests::quest_sign_up::quest_sign_up(context, interaction, framework.user_data, args)
                .await?;
        }
        "quest-sign-out" => {
            quests::quest_sign_out::quest_sign_out(context, interaction, framework.user_data)
                .await?;
        }
        "quest-add-random-participants" => {
            quests::quest_add_random_participants::quest_add_random_participants(
                context,
                interaction,
                framework.user_data,
            )
            .await?;
        }
        "quest-list-all-participants" => {
            quests::quest_list_all_participants::quest_list_all_participants(
                context,
                interaction,
                framework.user_data,
            )
            .await?;
        }
        "quest-history" => {
            return post_quest_history(context, &framework, interaction, args).await;
        }
        "ce" => {
            character_stat_edit::handle_character_editor_command(
                context,
                interaction,
                &framework.user_data.database,
                get_game_data(&framework, interaction).await,
                args,
            )
            .await?;
        }
        &_ => {}
    }

    Ok(())
}

async fn post_quest_history(
    context: &Context,
    framework: &FrameworkContext<'_>,
    interaction: &&ComponentInteraction,
    args: Vec<&str>,
) -> Result<(), Error> {
    let Ok(character_id) = i64::from_str(args[0]) else {
        return Err(Box::new(
            CommandInvocationError::new(&format!("Invalid character ID in request: {}", args[0]))
                .should_be_logged(),
        ));
    };

    match sqlx::query!(
        "SELECT quest_id FROM quest_completion WHERE character_id = ?",
        character_id
    )
    .fetch_all(&framework.user_data.database)
    .await
    {
        Ok(records) => {
            if records.is_empty() {
                let _ = send_ephemeral_reply(
                    interaction,
                    context,
                    "Seems like this character hasn't completed any quests yet!",
                )
                .await;
                return Ok(());
            }

            let mut result = String::from("### Quest History\n");

            for x in records {
                let channel_id = serenity::model::id::ChannelId::from(x.quest_id as u64);
                result.push_str(&channel_id.channel_id_link());
                result.push('\n');
            }

            result.push_str("\n(If any of these say 'Unknown', you need to tap on them to load the channel name. That's just how Discord works, sorry!)");

            for message in message_splitting::split_long_messages(result) {
                let _ = send_ephemeral_reply(interaction, context, &message).await;
            }

            Ok(())
        }
        Err(_) => Ok(()),
    }
}

async fn disable_button_on_original_message(
    context: &Context,
    interaction: &&ComponentInteraction,
) -> serenity::Result<()> {
    interaction
        .create_response(
            context,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new().components(
                    create_components_from_discord_components(
                        &interaction.message.components,
                        &interaction.data.custom_id,
                    ),
                ),
            ),
        )
        .await
}

fn create_components_from_discord_components(
    original_components: &Vec<ActionRow>,
    used_button_id: &String,
) -> Vec<CreateActionRow> {
    let mut result = Vec::new();

    for row in original_components {
        result.push(create_action_row_from_discord_components(
            row,
            used_button_id,
        ));
    }

    result
}

fn create_action_row_from_discord_components(
    row: &ActionRow,
    used_button_id: &String,
) -> CreateActionRow {
    let mut row_components = Vec::new();

    for component in &row.components {
        match component {
            ActionRowComponent::Button(button) => {
                row_components.push(create_button_from_discord_button(button, used_button_id));
            }
            ActionRowComponent::InputText(_) => todo!(),
            ActionRowComponent::SelectMenu(_) => todo!(),
            _ => todo!(),
        }
    }

    CreateActionRow::Buttons(row_components)
}

fn create_button_from_discord_button(button: &Button, used_button_id: &String) -> CreateButton {
    match &button.data {
        ButtonKind::Link { .. } => {
            todo!()
        }
        ButtonKind::NonLink { custom_id, style } => {
            let mut result = CreateButton::new(custom_id)
                .style(style.clone())
                .disabled(button.disabled || custom_id == used_button_id);

            if let Some(label) = &button.label {
                result = result.label(label);
            }

            result
        }
        ButtonKind::Premium { .. } => {
            todo!()
        }
    }
}
