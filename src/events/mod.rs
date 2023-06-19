mod button_interaction;
mod role_reaction;

use poise::{Event};
use serenity::client::Context;
use serenity::model::application::component::{ComponentType};
use serenity::model::application::interaction::{Interaction};
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use crate::{Error};
use crate::data::Data;

type FrameworkContext<'a> = poise::FrameworkContext<'a, Data, Error>;

pub async fn handle_events<'a>(
    context: &'a Context,
    event: &'a Event<'a>,
    framework: FrameworkContext<'a>
) -> Result<(), Error> {
    match event {
        Event::InteractionCreate {interaction} => handle_interaction(context, framework, interaction).await,
        Event::ReactionAdd {add_reaction} => role_reaction::handle_reaction_add(context, framework, add_reaction).await,
        Event::ReactionRemove {removed_reaction} => role_reaction::handle_reaction_remove(context, framework, removed_reaction).await,
        _ => Ok(())
    }
}

async fn handle_interaction(context: &Context, framework: FrameworkContext<'_>, interaction: &Interaction) -> Result<(), Error> {
    match interaction {
        Interaction::MessageComponent(component) => handle_message_component_interaction(context, framework, component).await,
        _ => Ok(())
    }
}

async fn handle_message_component_interaction(context: &Context, framework: FrameworkContext<'_>, interaction: &MessageComponentInteraction) -> Result<(), Error> {
    match interaction.data.component_type {
        ComponentType::ActionRow => {}
        ComponentType::Button => button_interaction::handle_button_interaction(context, framework, &interaction).await?,
        ComponentType::SelectMenu => {}
        ComponentType::InputText => {}
        ComponentType::Unknown => {}
        _ => {}
    }

    Ok(())
}

