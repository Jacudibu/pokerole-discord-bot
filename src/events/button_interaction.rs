use poise::CreateReply;
use serenity::builder::{CreateActionRow, CreateButton, CreateComponents};
use serenity::client::Context;
use serenity::model::application::component::{ActionRowComponent};
use serenity::model::application::interaction::{InteractionResponseType};
use serenity::model::prelude::component::{ActionRow, Button};
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use crate::{commands, Error, helpers};
use crate::commands::{efficiency, learns};
use crate::events::FrameworkContext;

pub async fn handle_button_interaction(context: &Context, framework: FrameworkContext<'_>, interaction: &&MessageComponentInteraction) -> Result<(), Error> {
    if interaction.data.custom_id.is_empty() {
        return Ok(());
    }

    let (command, args) = parse_command(interaction.data.custom_id.as_str());
    match command {
        "metronome" => {
            disable_button_on_original_message(context, interaction).await?;
            interaction.message.reply(context, commands::metronome::get_metronome_text(&framework.user_data.game)).await?;
        },
        "learns-all" => {
            disable_button_on_original_message(context, interaction).await?;
            let pokemon = framework.user_data.game.pokemon.get(args[0]).unwrap();
            for response_part in helpers::split_long_messages(pokemon.build_all_learnable_moves_list().into()) {
                interaction.message.reply(context, response_part).await?;
            }
        },
        "efficiency" => {
            disable_button_on_original_message(context, interaction).await?;
            let pokemon = framework.user_data.game.pokemon.get(args[0]).unwrap();
            interaction.message.reply(context, efficiency::get_type_resistances_string(pokemon, &framework.user_data.game.type_efficiency)).await?;
        },
        "moves" => {
            disable_button_on_original_message(context, interaction).await?;
            let pokemon = framework.user_data.game.pokemon.get(args[0]).unwrap();
            let mut create_reply = CreateReply::default();
            learns::create_reply(&mut create_reply, pokemon);
            interaction.create_followup_message(context, |f| {
                create_reply.to_slash_followup_response(f);
                f
            }).await?;
        },
        "abilities" => {
            disable_button_on_original_message(context, interaction).await?;
            let pokemon = framework.user_data.game.pokemon.get(args[0]).unwrap();
            interaction.message.reply(context, pokemon.build_ability_string(&framework.user_data.game.abilities).into()).await?;
        }
        &_ => {}
    }

    Ok(())
}

fn parse_command(custom_id: &str) -> (&str, Vec<&str>) {
    let mut split = custom_id.split('_');
    let command = split.next();
    let args: Vec<&str> = split.collect();

    (command.expect("Commands should never be empty at this point!"), args)
}

async fn disable_button_on_original_message(context: &Context, interaction: &&MessageComponentInteraction) -> serenity::Result<()> {
    interaction.create_interaction_response(context, |f| f
        .kind(InteractionResponseType::UpdateMessage)
        .interaction_response_data(|create_data| {
            create_data.set_components(create_components(&interaction.message.components, &interaction.data.custom_id))
        })
    ).await
}

fn create_components(original_components: &Vec<ActionRow>, used_button_id: &String) -> CreateComponents {
    let mut result = CreateComponents::default();

    for row in original_components {
        result.add_action_row(create_action_row(row, used_button_id));
    }

    result
}

fn create_action_row(row: &ActionRow, used_button_id: &String) -> CreateActionRow {
    let mut result = CreateActionRow::default();

    for component in &row.components {
        match component {
            ActionRowComponent::Button(button) => {
                result.add_button(create_button(button, used_button_id));
            }
            ActionRowComponent::SelectMenu(_) => todo!(),
            ActionRowComponent::InputText(_) => todo!(),
            _ => todo!(),
        }
    }

    result
}

fn create_button(button: &Button, used_button_id: &String) -> CreateButton {
    let mut result = CreateButton::default();

    result.style(button.style);

    if let Some(label) = &button.label {
        result.label(label);
    }
    if let Some(custom_id) = &button.custom_id {
        result.custom_id(custom_id);
        result.disabled(button.disabled || used_button_id == custom_id);
    } else {
        result.disabled(button.disabled);
    }

    result
}
