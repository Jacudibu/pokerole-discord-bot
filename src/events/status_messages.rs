use crate::data::Data;
use crate::helpers;
use log::info;
use serenity::all::{ChannelId, Context, CreateMessage};

pub async fn restart_message(ctx: &Context, data: &Data) {
    let Ok(status_channel_id) = std::env::var("STATUS_CHANNEL_ID") else {
        info!("STATUS_CHANNEL_ID is not defined, so we won't post any status updates.");
        return;
    };

    let Ok(status_channel_id) = status_channel_id.parse() else {
        return;
    };

    let status_channel_id = ChannelId::new(status_channel_id);

    let mut issue_summary = String::new();
    if let Some(issues) = &data.game.base_data.issues {
        issue_summary.push_str(&format!("**Base Data**:\n{}\n", issues));
    }
    for (_, data) in data.game.custom_data.iter() {
        if let Some(issues) = &data.issues {
            issue_summary.push_str(&format!("**{}**:\n{}\n", data.name, issues));
        }
    }

    let message = format!(
        "## The Bot just restarted.\n{}",
        if issue_summary.is_empty() {
            String::from("*Yay, all data looks good! Enjoy the day!~*")
        } else {
            format!("### Some Data Issues where detected:\n{issue_summary}")
        }
    );

    for message in helpers::split_long_messages(message) {
        let _ = status_channel_id
            .send_message(ctx, CreateMessage::new().content(message))
            .await;
    }
}
