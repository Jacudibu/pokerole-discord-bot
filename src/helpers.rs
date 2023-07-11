use crate::data::Data;
use crate::enums::QuestParticipantSelectionMechanism;
use crate::Error;
use serenity::builder::{CreateButton, CreateComponents};
use serenity::model::application::component::ButtonStyle;

pub fn create_styled_button(
    label: &str,
    custom_id: &str,
    is_disabled: bool,
    style: ButtonStyle,
) -> CreateButton {
    let mut button = create_button(label, custom_id, is_disabled);
    button.style(style);
    button
}

pub fn create_button(label: &str, custom_id: &str, is_disabled: bool) -> CreateButton {
    let mut button = CreateButton::default();
    button.label(label);
    button.custom_id(custom_id);
    button.style(ButtonStyle::Primary);
    button.disabled(is_disabled);
    button
}

pub fn split_long_messages(message: String) -> Vec<String> {
    if message.len() < 2000 {
        return vec![message];
    }

    let mut remaining = message.as_str();
    let mut result = Vec::default();
    while remaining.len() > 2000 {
        let split_index = find_best_split_pos(remaining);
        let split = remaining.split_at(split_index);

        result.push(split.0.to_string());
        remaining = split.1;
    }
    result.push(remaining.to_string());

    result
}

const MIN_SIZE: usize = 500;
fn find_best_split_pos(message: &str) -> usize {
    let split = message.split_at(2000).0;
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
    if let Some(index) = split.rfind('\n') {
        return index;
    }

    2000
}

struct Signup {
    character_name: String,
    user_id: i64,
    accepted: bool,
}

pub async fn generate_quest_post_message_content(
    data: &Data,
    channel_id: i64,
    maximum_participants: i64,
    selection_mechanism: QuestParticipantSelectionMechanism,
) -> Result<String, Error> {
    let quest_signups = sqlx::query_as!(
        Signup,
        "SELECT character.name as character_name, character.user_id as user_id, quest_signup.accepted as accepted
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

    let mut text = String::new();

    if !quest_signups.is_empty() {
        let mut accepted_participants: Vec<&Signup> = quest_signups
            .iter()
            .filter(|x| x.accepted)
            .collect::<Vec<&Signup>>();
        let mut floating_participants: Vec<&Signup> = quest_signups
            .iter()
            .filter(|x| !x.accepted)
            .collect::<Vec<&Signup>>();

        match selection_mechanism {
            QuestParticipantSelectionMechanism::FirstComeFirstServe => {
                let mut i = 0;
                while i < maximum_participants && !floating_participants.is_empty() {
                    accepted_participants.push(floating_participants.remove(0));
                    i += 1;
                }

                text.push_str("**Participants:**\n");
                add_character_names(&mut text, accepted_participants);

                if !floating_participants.is_empty() {
                    text.push_str("\n**Waiting Queue:**\n");
                    add_character_names(&mut text, floating_participants);
                }
            }
            QuestParticipantSelectionMechanism::Random
            | QuestParticipantSelectionMechanism::GMPicks => {
                if accepted_participants.is_empty() {
                    text.push_str("**Signups:**\n");
                    add_character_names(&mut text, floating_participants);
                } else {
                    text.push_str("**Participants:**\n");
                    add_character_names(&mut text, accepted_participants);
                    text.push_str("\n**Waiting Queue:**\n");
                    add_character_names(&mut text, floating_participants);
                }
            }
        }
    }

    text.push_str(
        format!(
            "\nParticipant Selection Method: **{}**\nMaximum Participants: **{}**",
            selection_mechanism, maximum_participants,
        )
        .as_str(),
    );
    text.push_str("\n**Use the buttons below to sign up!**");
    Ok(text)
}

fn add_character_names(text: &mut String, quest_signups: Vec<&Signup>) {
    for record in quest_signups {
        text.push_str(format!("- {} (<@{}>)\n", record.character_name, record.user_id).as_str());
    }
}

pub fn create_quest_signup_buttons(components: &mut CreateComponents) -> &mut CreateComponents {
    components.create_action_row(|action_row| {
        action_row
            .add_button(create_styled_button(
                "Sign up!",
                "quest-sign-up",
                false,
                ButtonStyle::Success,
            ))
            .add_button(create_styled_button(
                "Sign out",
                "quest-sign-out",
                false,
                ButtonStyle::Danger,
            ))
    })
}
