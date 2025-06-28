use crate::shared::data::Data;
use crate::shared::emoji;
use crate::shared::enums::QuestParticipantSelectionMechanism;
use crate::shared::utility::button_building;
use crate::shared::utility::channel_id_ext::ChannelIdExt;
use crate::Error;
use serenity::all::{ButtonStyle, ChannelId, Context, CreateActionRow, EditMessage, MessageId};

struct QuestSignup {
    character_name: String,
    character_experience: i64,
    stat_channel_id: ChannelId,
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
    let records = sqlx::query!("
SELECT character.id as character_id, character.name as character_name, character.user_id as user_id,
       character.species_api_id as character_species_id, character.experience as character_experience,
       character.stat_channel_id as stat_channel_id, quest_signup.accepted as accepted
FROM quest_signup
INNER JOIN character ON
    quest_signup.character_id = character.id
WHERE quest_id = ?
ORDER BY quest_signup.accepted DESC, quest_signup.timestamp
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

        quest_signups.push(QuestSignup {
            character_name: record.character_name.clone(),
            character_experience: record.character_experience,
            stat_channel_id: ChannelId::new(record.stat_channel_id as u64),
            user_id: record.user_id,
            accepted: record.accepted,
            emoji,
        });
    }

    let mut text = String::new();
    let mut hidden_signup_count = 0;

    if !quest_signups.is_empty() {
        let mut accepted_participants: Vec<&QuestSignup> = quest_signups
            .iter()
            .filter(|x| x.accepted)
            .collect::<Vec<&QuestSignup>>();

        let mut floating_participants: Vec<&QuestSignup> = quest_signups
            .iter()
            .filter(|x| !x.accepted)
            .collect::<Vec<&QuestSignup>>();

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
                add_character_names(&mut text, accepted_participants, displayable_accepted, true);

                if !floating_participants.is_empty() {
                    text.push_str("\n**Waiting Queue:**\n");
                    add_character_names(
                        &mut text,
                        floating_participants,
                        displayable_floating,
                        false,
                    );
                }
            }
            QuestParticipantSelectionMechanism::Random
            | QuestParticipantSelectionMechanism::GMPicks => {
                if accepted_participants.is_empty() {
                    text.push_str("**Signups:**\n");
                    add_character_names(
                        &mut text,
                        floating_participants,
                        displayable_accepted,
                        false,
                    );
                } else {
                    text.push_str("**Participants:**\n");
                    add_character_names(
                        &mut text,
                        accepted_participants,
                        displayable_accepted,
                        true,
                    );
                    if !floating_participants.is_empty() {
                        text.push_str("\n**Waiting Queue:**\n");
                        add_character_names(
                            &mut text,
                            floating_participants,
                            displayable_floating,
                            false,
                        );
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

fn add_character_names(
    text: &mut String,
    quest_signups: Vec<&QuestSignup>,
    max: usize,
    display_link_to_character_sheet: bool,
) {
    for record in quest_signups.iter().take(max) {
        let cs_link = if display_link_to_character_sheet {
            format!(" ({})", record.stat_channel_id.channel_id_link())
        } else {
            String::new()
        };

        text.push_str(
            format!(
                "- {}{} (<@{}>) Lv.{}{cs_link}\n",
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
        button_building::create_styled_button(
            "Sign up!",
            "quest-sign-up",
            false,
            ButtonStyle::Success,
        ),
        button_building::create_styled_button(
            "Sign out",
            "quest-sign-out",
            false,
            ButtonStyle::Danger,
        ),
    ];

    if signup_mechanism == QuestParticipantSelectionMechanism::Random {
        buttons.push(button_building::create_styled_button(
            "Select Random Participants",
            "quest-add-random-participants",
            false,
            ButtonStyle::Secondary,
        ));
    }

    if too_many_arguments {
        vec![
            CreateActionRow::Buttons(buttons),
            CreateActionRow::Buttons(vec![button_building::create_styled_button(
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
