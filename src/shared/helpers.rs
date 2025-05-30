use crate::shared::data::Data;
use crate::shared::enums::QuestParticipantSelectionMechanism;
use crate::shared::game_data::pokemon::Pokemon;
use crate::shared::game_data::{GameData, PokemonApiId};
use crate::shared::{constants, emoji};
use crate::Error;
use log::error;
use regex::Regex;
use serenity::all::{
    ButtonStyle, ChannelId, Context, CreateActionRow, CreateButton, CreateMessage, EditMessage,
    EditThread, HttpError, Message, MessageId,
};

pub fn create_styled_button(
    label: &str,
    custom_id: &str,
    is_disabled: bool,
    style: ButtonStyle,
) -> CreateButton {
    create_button(label, custom_id, is_disabled).style(style)
}

pub fn create_button(label: &str, custom_id: &str, is_disabled: bool) -> CreateButton {
    CreateButton::new(custom_id)
        .label(label)
        .style(ButtonStyle::Primary)
        .disabled(is_disabled)
}

pub fn split_long_messages(message: String) -> Vec<String> {
    split_long_messages_with_custom_max_length(message, constants::DISCORD_MESSAGE_LENGTH_LIMIT)
}

pub fn split_long_messages_with_custom_max_length(
    message: String,
    max_length: usize,
) -> Vec<String> {
    if message.len() < max_length {
        return vec![message];
    }

    let mut remaining = message.as_str();
    let mut result = Vec::default();
    while remaining.len() > max_length {
        let split_index = find_best_split_pos(remaining, max_length);
        let split = remaining.split_at(split_index);

        result.push(split.0.to_string());
        remaining = split.1.trim_start();
    }
    result.push(remaining.to_string());

    result
}

const MIN_SIZE: usize = 500;

fn find_best_split_pos(message: &str, max_length: usize) -> usize {
    let split = message.split_at(max_length).0;
    if let Some(index) = split.rfind("\n# ") {
        if index > MIN_SIZE {
            return index;
        }
    }
    if let Some(index) = split.rfind("\n## ") {
        if index > MIN_SIZE {
            return index;
        }
    }
    if let Some(index) = split.rfind("\n### ") {
        if index > MIN_SIZE {
            return index;
        }
    }
    if let Some(index) = split.rfind("\n**") {
        return index;
    }
    if let Some(index) = split.rfind("\n\n") {
        return index;
    }
    if let Some(index) = split.rfind("\n- ") {
        return index;
    }
    if let Some(index) = split.rfind('\n') {
        return index;
    }

    max_length
}

struct Signup {
    character_name: String,
    character_experience: i64,
    user_id: i64,
    accepted: bool,
    emoji: String,
}

const MAX_SIGNUP_DISPLAY_COUNT: usize = 18;

pub async fn generate_quest_post_message_content(
    context: &Context,
    data: &Data,
    channel_id: i64,
    maximum_participants: i64,
    selection_mechanism: QuestParticipantSelectionMechanism,
) -> Result<(String, bool), Error> {
    let (mut text, too_many_signups) = create_quest_participant_list(
        context,
        data,
        channel_id,
        maximum_participants,
        selection_mechanism,
        true,
    )
    .await?;

    text.push_str(
        format!(
            "\nParticipant Selection Method: **{:?}**\nMaximum Participants: **{}**",
            selection_mechanism, maximum_participants,
        )
        .as_str(),
    );
    text.push_str("\n**Use the buttons below to sign up!**");
    Ok((text, too_many_signups))
}

pub async fn create_quest_participant_list(
    context: &Context,
    data: &Data,
    channel_id: i64,
    maximum_participants: i64,
    selection_mechanism: QuestParticipantSelectionMechanism,
    stop_at_character_limit: bool,
) -> Result<(String, bool), Error> {
    let records = sqlx::query!(
        "SELECT character.id as character_id, character.name as character_name, character.user_id as user_id, character.species_api_id as character_species_id, character.experience as character_experience, quest_signup.accepted as accepted
FROM quest_signup
INNER JOIN character ON
    quest_signup.character_id = character.id
WHERE quest_id = ?
ORDER BY quest_signup.accepted DESC, quest_signup.timestamp ASC
",
        channel_id
    )
        .fetch_all(&data.database)
        .await?;

    let mut quest_signups = Vec::new();
    for record in records {
        let emoji = match emoji::get_character_emoji(context, data, record.character_id).await {
            Some(emoji) => format!("{} ", emoji),
            None => String::new(),
        };

        quest_signups.push(Signup {
            character_name: record.character_name.clone(),
            character_experience: record.character_experience,
            user_id: record.user_id,
            accepted: record.accepted,
            emoji,
        });
    }

    let mut text = String::new();
    let mut hidden_signup_count = 0;

    if !quest_signups.is_empty() {
        let mut accepted_participants: Vec<&Signup> = quest_signups
            .iter()
            .filter(|x| x.accepted)
            .collect::<Vec<&Signup>>();

        let mut floating_participants: Vec<&Signup> = quest_signups
            .iter()
            .filter(|x| !x.accepted)
            .collect::<Vec<&Signup>>();

        let (displayable_accepted, displayable_floating) = if stop_at_character_limit {
            (
                MAX_SIGNUP_DISPLAY_COUNT,
                MAX_SIGNUP_DISPLAY_COUNT
                    - accepted_participants.len().min(MAX_SIGNUP_DISPLAY_COUNT),
            )
        } else {
            (usize::MAX, usize::MAX)
        };

        if quest_signups.len() > MAX_SIGNUP_DISPLAY_COUNT {
            hidden_signup_count = quest_signups.len() - MAX_SIGNUP_DISPLAY_COUNT;
        }

        match selection_mechanism {
            QuestParticipantSelectionMechanism::FirstComeFirstServe => {
                let mut i = 0;
                while i < maximum_participants && !floating_participants.is_empty() {
                    accepted_participants.push(floating_participants.remove(0));
                    i += 1;
                }

                text.push_str("**Participants:**\n");
                add_character_names(&mut text, accepted_participants, displayable_accepted);

                if !floating_participants.is_empty() {
                    text.push_str("\n**Waiting Queue:**\n");
                    add_character_names(&mut text, floating_participants, displayable_floating);
                }
            }
            QuestParticipantSelectionMechanism::Random
            | QuestParticipantSelectionMechanism::GMPicks => {
                if accepted_participants.is_empty() {
                    text.push_str("**Signups:**\n");
                    add_character_names(&mut text, floating_participants, displayable_accepted);
                } else {
                    text.push_str("**Participants:**\n");
                    add_character_names(&mut text, accepted_participants, displayable_accepted);
                    if !floating_participants.is_empty() {
                        text.push_str("\n**Waiting Queue:**\n");
                        add_character_names(&mut text, floating_participants, displayable_floating);
                    }
                }
            }
        }

        if stop_at_character_limit && hidden_signup_count > 0 {
            text.push_str(&format!(
                "- **And {hidden_signup_count} more!** Press the button below to see all.\n"
            ));
        }
    }
    Ok((text, hidden_signup_count > 0))
}

fn add_character_names(text: &mut String, quest_signups: Vec<&Signup>, max: usize) {
    for record in quest_signups.iter().take(max) {
        text.push_str(
            format!(
                "- {}{} (<@{}>) Lv.{}\n",
                record.emoji,
                record.character_name,
                record.user_id,
                1 + record.character_experience / 100,
            )
            .as_str(),
        );
    }
}

pub fn create_quest_signup_buttons(
    signup_mechanism: QuestParticipantSelectionMechanism,
    too_many_arguments: bool,
) -> Vec<CreateActionRow> {
    let mut buttons = vec![
        create_styled_button("Sign up!", "quest-sign-up", false, ButtonStyle::Success),
        create_styled_button("Sign out", "quest-sign-out", false, ButtonStyle::Danger),
    ];

    if signup_mechanism == QuestParticipantSelectionMechanism::Random {
        buttons.push(create_styled_button(
            "Select Random Participants",
            "quest-add-random-participants",
            false,
            ButtonStyle::Secondary,
        ));
    }

    if too_many_arguments {
        vec![
            CreateActionRow::Buttons(buttons),
            CreateActionRow::Buttons(vec![create_styled_button(
                "Show all participants!",
                "quest-list-all-participants",
                false,
                ButtonStyle::Primary,
            )]),
        ]
    } else {
        vec![CreateActionRow::Buttons(buttons)]
    }
}

pub async fn update_quest_message(
    context: &Context,
    data: &Data,
    channel_id: i64,
) -> Result<(), Error> {
    let quest_record = sqlx::query!(
        "SELECT bot_message_id, maximum_participant_count, participant_selection_mechanism FROM quest WHERE channel_id = ?",
        channel_id
    )
        .fetch_one(&data.database)
        .await?;

    let selection_mechanism =
        QuestParticipantSelectionMechanism::from_repr(quest_record.participant_selection_mechanism)
            .expect("Should always be valid!");

    let (text, too_many_signups) = generate_quest_post_message_content(
        context,
        data,
        channel_id,
        quest_record.maximum_participant_count,
        selection_mechanism,
    )
    .await?;

    let message = context
        .http
        .get_message(
            ChannelId::new(channel_id as u64),
            MessageId::new(quest_record.bot_message_id as u64),
        )
        .await;
    if let Ok(mut message) = message {
        message
            .edit(
                context,
                EditMessage::new()
                    .content(text)
                    .components(create_quest_signup_buttons(
                        selection_mechanism,
                        too_many_signups,
                    )),
            )
            .await?;
    }
    Ok(())
}

pub fn calculate_available_combat_points(level: i64) -> i64 {
    level + 3
}

const STAGE1_EVOLUTION_LEVEL_THRESHOLD: i64 = 3;
const STAGE2_EVOLUTION_LEVEL_THRESHOLD: i64 = 6;

pub fn get_usual_evolution_stage_for_level<'a>(
    level: i64,
    pokemon: &'a Pokemon,
    game_data: &'a GameData,
    stat_override: Option<i64>,
) -> &'a Pokemon {
    if let Some(stat_override) = stat_override {
        let api_id = PokemonApiId(stat_override as u16);
        return game_data.pokemon_by_api_id.get(&api_id).unwrap();
    }

    if pokemon.evolves_from.is_none() {
        return pokemon;
    }
    let evolves_from = pokemon.evolves_from.unwrap();

    if level >= STAGE2_EVOLUTION_LEVEL_THRESHOLD {
        return pokemon;
    }

    let pre_evolution = game_data
        .pokemon_by_api_id
        .get(&evolves_from)
        .expect("Pre-Evolutions should be implemented!");

    if pre_evolution.evolves_from.is_none() {
        // Confirmed one stage evo
        return if level >= STAGE1_EVOLUTION_LEVEL_THRESHOLD {
            pokemon
        } else {
            pre_evolution
        };
    }

    // Confirmed two stage evo
    if level >= STAGE1_EVOLUTION_LEVEL_THRESHOLD {
        return pre_evolution;
    }

    // Confirmed first stage stats
    game_data
        .pokemon_by_api_id
        .get(&pre_evolution.evolves_from.unwrap())
        .expect("Pre-Evolutions should be implemented!")
}

pub fn calculate_level_from_experience(experience: i64) -> i64 {
    experience / 100 + 1
}

pub fn calculate_current_experience(experience: i64) -> i64 {
    experience % 100
}

pub fn channel_id_link(channel_id: ChannelId) -> String {
    format!("<#{}>", channel_id)
}

pub fn calculate_next_limit_break_cost(limit_break_count: i64) -> i64 {
    2 + limit_break_count
}

pub async fn handle_error_during_message_edit(
    ctx: &Context,
    e: serenity::Error,
    mut message_to_edit: Message,
    updated_message_content: impl Into<String>,
    components: Option<Vec<CreateActionRow>>,
    name: impl Into<String>,
    reply_channel_id: Option<ChannelId>,
) {
    if let serenity::Error::Http(HttpError::UnsuccessfulRequest(e)) = &e {
        if e.error.code == constants::discord_error_codes::ARCHIVED_THREAD {
            if let Ok(channel) = message_to_edit.channel(ctx).await {
                if let Some(mut channel) = channel.guild() {
                    match channel
                        .edit_thread(ctx, EditThread::new().archived(false))
                        .await
                    {
                        Ok(_) => {
                            let mut edit_message =
                                EditMessage::new().content(updated_message_content);
                            if let Some(components) = components {
                                edit_message = edit_message.components(components);
                            }

                            if let Err(e) = message_to_edit.edit(ctx, edit_message).await {
                                let _ = log_error(ctx, format!(
                                    "**Failed to update the stat message for {}!**.\nThe change has been tracked, but whilst updating the message some error occurred:\n```{:?}```\n",
                                    name.into(),
                                    e,
                                )).await;
                            }
                        }
                        Err(e) => {
                            let name = name.into();
                            log_error(ctx, format!(
                                "Some very random error occurred when updating the stat message for {}.\n**The requested change has been applied, but it isn't shown in the message there right now.**\n Error:\n```{:?}```",
                                &name, e)
                            ).await;
                            if let Some(reply_channel_id) = reply_channel_id {
                                let _ = reply_channel_id.say(ctx, &format!(
                                    "Some very random error occurred when updating the stat message for {}.\n**The requested change has been applied, but it isn't shown in the message there right now.**\n*This has been logged.*",
                                    &name)).await;
                            }
                        }
                    }

                    return;
                }
            }
        }
    }

    let name = name.into();
    log_error(ctx, format!(
        "Some very random error occurred when updating the stat message for {}.\n**The requested change has been applied, but it isn't shown in the message there right now.**\n Error:\n```{:?}```",
        &name, e)).await;
    if let Some(reply_channel_id) = reply_channel_id {
        let _ = reply_channel_id.say(ctx, &format!(
            "Some very random error occurred when updating the stat message for {}.\n**The requested change has been applied, but it isn't shown in the message there right now.**\n*This has been logged.*",
            &name)).await;
    }
}

pub async fn log_error(context: &Context, text: impl Into<String>) {
    let string = text.into();
    error!("{}", string);
    let _ = constants::ERROR_LOG_CHANNEL
        .send_message(context, CreateMessage::new().content(string))
        .await;
}

pub fn validate_user_input<'a>(text: &str, max_length: usize) -> Result<(), &'a str> {
    if text.len() > max_length {
        return Err("Input string too long!");
    }

    let regex = Regex::new(r"^[\w ']*$").unwrap();
    if regex.is_match(text) {
        Ok(())
    } else {
        Err("Failed to validate input string!")
    }
}
