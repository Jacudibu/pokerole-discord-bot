use crate::commands::{Context, Error};

struct Quest {
    channel_id: i64,
    creator_id: i64,
    completion_timestamp: i64,
    character_ids: Vec<i64>,
}

/// Manually remove a character to the quest.
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn migrate_db_to_new_quest_system(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id: i64 = 1113123066059436093;
    let quests = vec![
        Quest {
            channel_id: 1121106350571790508,
            creator_id: 878982444412448829,
            completion_timestamp: 1687539900,
            character_ids: vec![14, 15, 23, 16, 5],
        },
        Quest {
            channel_id: 1123009677068869812,
            creator_id: 243777734168281089,
            completion_timestamp: 1687846920,
            character_ids: vec![21, 10, 24, 11],
        },
        Quest {
            channel_id: 1124914136925606009,
            creator_id: 243777734168281089,
            completion_timestamp: 1688304720,
            character_ids: vec![15, 23, 29, 24],
        },
        Quest {
            channel_id: 1126293850189729803,
            creator_id: 399228420330094592,
            completion_timestamp: 1688891580,
            character_ids: vec![14, 16, 24, 7],
        },
    ];

    for x in quests {
        sqlx::query!(
            "INSERT INTO quest (guild_id,
channel_id,
creator_id,
bot_message_id,
creation_timestamp,
completion_timestamp,
maximum_participant_count,
participant_selection_mechanism) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            guild_id,
            x.channel_id,
            x.creator_id,
            0,
            x.completion_timestamp,
            x.completion_timestamp,
            4,
            1
        )
        .execute(&ctx.data().database)
        .await?;

        for character_id in x.character_ids {
            sqlx::query!(
                "INSERT INTO quest_signup (quest_id,
character_id,
timestamp,
accepted) VALUES (?, ?, ?, ?)",
                x.channel_id,
                character_id,
                x.completion_timestamp,
                true,
            )
            .execute(&ctx.data().database)
            .await?;

            sqlx::query!(
                "INSERT INTO quest_completion (quest_id, character_id) VALUES (?, ?)",
                x.channel_id,
                character_id,
            )
            .execute(&ctx.data().database)
            .await?;
        }
    }

    ctx.say("ok.").await?;
    Ok(())
}
