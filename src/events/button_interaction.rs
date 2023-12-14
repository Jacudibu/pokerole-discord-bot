use crate::commands::{efficiency, learns};
use crate::events::{parse_interaction_command, quests, FrameworkContext};
use crate::{commands, helpers, Error};
use serenity::all::{
    ActionRow, ActionRowComponent, Button, ButtonKind, ComponentInteraction,
    CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};
use serenity::builder::{CreateActionRow, CreateButton};
use serenity::client::Context;

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
            interaction
                .message
                .reply(
                    context,
                    commands::metronome::get_metronome_text(&framework.user_data.game),
                )
                .await?;
        }
        "learns-all" => {
            disable_button_on_original_message(context, interaction).await?;
            let pokemon = framework.user_data.game.pokemon.get(args[0]).unwrap();
            for response_part in
                helpers::split_long_messages(pokemon.build_all_learnable_moves_list().into())
            {
                interaction.message.reply(context, response_part).await?;
            }
        }
        "efficiency" => {
            disable_button_on_original_message(context, interaction).await?;
            let pokemon = framework.user_data.game.pokemon.get(args[0]).unwrap();
            interaction
                .message
                .reply(
                    context,
                    efficiency::get_type_resistances_string(
                        pokemon,
                        &framework.user_data.game.type_efficiency,
                    ),
                )
                .await?;
        }
        "pokedex" => {
            disable_button_on_original_message(context, interaction).await?;
            let pokemon = framework.user_data.game.pokemon.get(args[0]).unwrap();
            for response_part in helpers::split_long_messages(pokemon.build_pokedex_string()) {
                interaction.message.reply(context, response_part).await?;
            }
        }
        "moves" => {
            disable_button_on_original_message(context, interaction).await?;
            let pokemon = framework.user_data.game.pokemon.get(args[0]).unwrap();
            interaction
                .create_followup(
                    context,
                    learns::create_reply(pokemon)
                        .to_slash_followup_response(CreateInteractionResponseFollowup::new()),
                )
                .await?;
        }
        "abilities" => {
            disable_button_on_original_message(context, interaction).await?;
            let pokemon = framework.user_data.game.pokemon.get(args[0]).unwrap();
            interaction
                .message
                .reply(
                    context,
                    pokemon
                        .build_ability_string(&framework.user_data.game.abilities)
                        .into(),
                )
                .await?;
        }
        "roll-dice" => {
            let message = commands::roll::parse_query(args[0])
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
        &_ => {}
    }

    Ok(())
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
    }
}
