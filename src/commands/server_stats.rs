use crate::commands::Error;
use crate::shared::game_data::{GameData, PokemonApiId};
use crate::shared::utility::message_splitting;
use crate::shared::{PoiseContext, SerenityContext, emoji};
use log::warn;
use sqlx::{Pool, Sqlite};

/// View some fancy server stats.
#[poise::command(slash_command, guild_only)]
pub async fn server_stats(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let defer = ctx.defer();
    let guild_id = ctx.guild_id().expect("Command is guild_only!").get() as i64;

    let mut result = String::from("# Server Stats\n");

    let quests = count_quests(guild_id, &ctx.data().database);
    let character_money = sum_character_money(guild_id, &ctx.data().database);
    let wallet_money = sum_wallet_money(guild_id, &ctx.data().database);
    let pokemon = count_pokemon(
        guild_id,
        ctx.serenity_context(),
        &ctx.data().database,
        ctx.data().game.get_by_context(&ctx).await,
    );

    let (quests, character_money, wallet_money, pokemon) =
        tokio::join!(quests, character_money, wallet_money, pokemon);

    result.push_str(&quests);
    result.push_str(&character_money);
    result.push_str(&wallet_money);
    result.push_str(&pokemon);

    result.push_str("\n*(Got any other ideas for what should be displayed here? Lemme know and I might add it!)*");

    let _ = defer.await;

    for message in message_splitting::split_long_messages(result) {
        let result = ctx.reply(message).await;
        if let Err(error) = result {
            let _ = ctx
                .reply(&format!(
                    "Encountered an unexpected error:\n```{}```",
                    error
                ))
                .await;
        }
    }

    Ok(())
}

async fn count_quests(guild_id: i64, database: &Pool<Sqlite>) -> String {
    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM quest WHERE guild_id = ? AND completion_timestamp IS NOT NULL", guild_id
    )
        .fetch_one(database)
        .await
        .unwrap();

    format!("- Completed Quests: {}\n", result.count)
}

async fn sum_character_money(guild_id: i64, database: &Pool<Sqlite>) -> String {
    let result = sqlx::query!(
        "SELECT SUM(money) as sum FROM character WHERE guild_id = ? AND is_retired = false",
        guild_id
    )
    .fetch_one(database)
    .await
    .unwrap();

    format!(
        "- Total character wealth: {} {}\n",
        result.sum.unwrap_or(0),
        emoji::POKE_COIN
    )
}

async fn sum_wallet_money(guild_id: i64, database: &Pool<Sqlite>) -> String {
    let result = sqlx::query!(
        "SELECT SUM(money) as sum FROM wallet WHERE guild_id = ?",
        guild_id
    )
    .fetch_one(database)
    .await
    .unwrap();

    format!(
        "- Total shop wealth: {} {}\n",
        result.sum.unwrap_or(0),
        emoji::POKE_COIN
    )
}

async fn count_pokemon(
    guild_id: i64,
    context: &SerenityContext,
    database: &Pool<Sqlite>,
    game_data: &GameData,
) -> String {
    let records = sqlx::query!(
        "SELECT species_api_id, COUNT(*) as count FROM character WHERE guild_id = ? AND is_retired = false GROUP BY species_api_id ORDER BY species_api_id",
        guild_id
    )
        .fetch_all(database)
        .await
        .unwrap();

    let mut message = String::default();
    message.push_str("### Played Species Overview\n");
    if records.is_empty() {
        message.push_str("None! Oh no.");
        return message;
    }

    for record in records {
        let species_api_id = PokemonApiId(record.species_api_id as u16);
        if let Some(pokemon) = game_data.pokemon_by_api_id.get(&species_api_id) {
            message.push_str(&format!(
                "- {}{}: {}\n",
                emoji::get_any_pokemon_emoji_with_space(context, database, pokemon).await,
                pokemon.name,
                record.count
            ));
        } else {
            warn!("Was unable to resolve species api id {}", species_api_id.0);
        }
    }

    message
}
