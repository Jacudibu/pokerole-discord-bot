use crate::data::Data;
use crate::enums::{MysteryDungeonRank, QuestParticipantSelectionMechanism};
use crate::{emoji, Error};
use serenity::all::{
    ButtonStyle, ChannelId, Context, CreateActionRow, CreateButton, EditMessage, MessageId,
};

pub const ADMIN_PING_STRING: &str = "<@878982444412448829>";
pub const ERROR_LOG_CHANNEL: ChannelId = ChannelId::new(1188864512439369779);

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
    character_experience: i64,
    user_id: i64,
    accepted: bool,
    emoji: String,
}

pub async fn generate_quest_post_message_content(
    data: &Data,
    channel_id: i64,
    maximum_participants: i64,
    selection_mechanism: QuestParticipantSelectionMechanism,
) -> Result<String, Error> {
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
        let emoji = match emoji::get_character_emoji(data, record.character_id).await {
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
                    if !floating_participants.is_empty() {
                        text.push_str("\n**Waiting Queue:**\n");
                        add_character_names(&mut text, floating_participants);
                    }
                }
            }
        }
    }

    text.push_str(
        format!(
            "\nParticipant Selection Method: **{:?}**\nMaximum Participants: **{}**",
            selection_mechanism, maximum_participants,
        )
        .as_str(),
    );
    text.push_str("\n**Use the buttons below to sign up!**");
    Ok(text)
}

fn add_character_names(text: &mut String, quest_signups: Vec<&Signup>) {
    for record in quest_signups {
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
) -> CreateActionRow {
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

    CreateActionRow::Buttons(buttons)
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

    let text = generate_quest_post_message_content(
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
                    .components(vec![create_quest_signup_buttons(selection_mechanism)]),
            )
            .await?;
    }
    Ok(())
}

pub fn calculate_available_social_points(rank: &MysteryDungeonRank) -> u8 {
    match rank {
        MysteryDungeonRank::Bronze => 4,
        MysteryDungeonRank::Silver => 4 + 2,
        MysteryDungeonRank::Gold => 4 + 4,
        MysteryDungeonRank::Platinum => 4 + 6,
        MysteryDungeonRank::Diamond => 4 + 8,
    }
}

pub fn calculate_available_combat_points(level: i64) -> i64 {
    level + 3
}
