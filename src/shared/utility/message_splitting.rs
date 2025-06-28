use crate::shared::constants;

const MIN_SIZE: usize = 500;

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
