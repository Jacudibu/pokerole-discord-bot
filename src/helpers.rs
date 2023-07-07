use crate::data::Data;
use crate::Error;
use serenity::builder::CreateButton;
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

pub async fn generate_quest_post_message_content(
    data: &Data,
    channel_id: i64,
) -> Result<String, Error> {
    let quest_signups = sqlx::query!(
        "SELECT character.name as character_name
FROM quest_signup
INNER JOIN character ON
    quest_signup.character_id = character.id
WHERE quest_id = ?
",
        channel_id
    )
    .fetch_all(&data.database)
    .await?;

    let mut text = String::new();

    if !quest_signups.is_empty() {
        text.push_str("**Signups:**\n");
        for record in quest_signups {
            text.push_str("- ");
            text.push_str(record.character_name.as_str());
            text.push('\n');
        }
    }

    text.push_str("\nUse the buttons below to sign up!");
    Ok(text)
}
