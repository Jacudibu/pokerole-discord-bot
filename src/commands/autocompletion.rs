use crate::commands::get_servers_this_user_is_active_in;
use crate::shared::enums::PokemonTypeWithoutShadow;
use crate::shared::PoiseContext;
use std::cmp::Ordering;

fn filter_and_sort<'a>(
    partial: &str,
    commands: impl Iterator<Item = &'a String>,
    minimum_query_length: usize,
) -> Vec<String> {
    if partial.len() < minimum_query_length {
        return Vec::default();
    }

    let lowercase_user_input = &partial.to_lowercase();
    let mut result: Vec<String> = commands
        .filter(move |x| x.to_lowercase().contains(lowercase_user_input))
        .cloned()
        .collect();

    result.sort_by(|a, b| {
        if a.to_lowercase().starts_with(lowercase_user_input) {
            if b.to_lowercase().starts_with(lowercase_user_input) {
                return a.cmp(b);
            }
            return Ordering::Less;
        }
        if b.to_lowercase().starts_with(lowercase_user_input) {
            return Ordering::Greater;
        }

        Ordering::Equal
    });

    result
}

pub async fn autocomplete_move<'a>(ctx: PoiseContext<'a>, partial: &'a str) -> Vec<String> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;
    filter_and_sort(partial, game_data.move_names.iter(), 2)
}

pub async fn autocomplete_ability<'a>(ctx: PoiseContext<'a>, partial: &'a str) -> Vec<String> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;
    filter_and_sort(partial, game_data.ability_names.iter(), 2)
}

pub async fn autocomplete_pokemon<'a>(ctx: PoiseContext<'a>, partial: &'a str) -> Vec<String> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;
    filter_and_sort(partial, game_data.pokemon_names.iter(), 2)
}

pub async fn autocomplete_item<'a>(ctx: PoiseContext<'a>, partial: &'a str) -> Vec<String> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;
    filter_and_sort(partial, game_data.item_names.iter(), 2)
}

pub async fn autocomplete_weather<'a>(ctx: PoiseContext<'a>, partial: &'a str) -> Vec<String> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;
    filter_and_sort(partial, game_data.weather_names.iter(), 0)
}

pub async fn autocomplete_status_effect<'a>(
    ctx: PoiseContext<'a>,
    partial: &'a str,
) -> Vec<String> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;
    filter_and_sort(partial, game_data.status_effects_names.iter(), 0)
}

pub async fn autocomplete_rule<'a>(ctx: PoiseContext<'a>, partial: &'a str) -> Vec<String> {
    let guild_id = ctx.guild_id().expect("Command should be guild_only!").get() as i64;
    let entries = sqlx::query!("SELECT name FROM guild_rules WHERE guild_id = ?", guild_id)
        .fetch_all(&ctx.data().database)
        .await;

    if let Ok(entries) = entries {
        filter_and_sort(partial, entries.iter().map(|x| &x.name), 0)
    } else {
        Vec::new()
    }
}

/// Lists names for servers the user has been registered on.
pub async fn autocomplete_server_name<'a>(ctx: PoiseContext<'a>, partial: &'a str) -> Vec<String> {
    let known_servers = get_servers_this_user_is_active_in(&ctx).await;

    if let Ok(entries) = known_servers {
        filter_and_sort(partial, entries.iter().map(|x| x.name.as_ref().unwrap()), 0)
    } else {
        Vec::new()
    }
}

pub async fn autocomplete_nature<'a>(ctx: PoiseContext<'a>, partial: &'a str) -> Vec<String> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;
    filter_and_sort(partial, game_data.nature_names.iter(), 0)
}

pub async fn autocomplete_potion<'a>(ctx: PoiseContext<'a>, partial: &'a str) -> Vec<String> {
    let game_data = ctx.data().game.get_by_context(&ctx).await;
    filter_and_sort(partial, game_data.potion_names.iter(), 0)
}

pub async fn autocomplete_pokemon_type<'a>(
    _ctx: PoiseContext<'a>,
    partial: &'a str,
) -> Vec<String> {
    filter_and_sort(partial, PokemonTypeWithoutShadow::get_names_vec().iter(), 0)
}

pub async fn autocomplete_character_name<'a>(
    ctx: PoiseContext<'a>,
    partial: &'a str,
) -> Vec<String> {
    let guild_id = ctx.guild_id().expect("Command should be guild_only!").get();

    filter_and_sort(
        partial,
        ctx.data()
            .cache
            .get_characters()
            .await
            .iter()
            .filter(|(_, cache_item)| cache_item.guild_id == guild_id)
            .filter(|(_, cache_item)| !cache_item.is_retired)
            .map(|(_, cache_item)| cache_item.get_autocomplete_name()),
        0,
    )
}

pub async fn autocomplete_wallet_name<'a>(ctx: PoiseContext<'a>, partial: &'a str) -> Vec<String> {
    let guild_id = ctx.guild_id().expect("Command should be guild_only!").get() as i64;
    let entries = sqlx::query!(
        "SELECT name FROM wallet WHERE wallet.guild_id = ?",
        guild_id
    )
    .fetch_all(&ctx.data().database)
    .await;

    if let Ok(entries) = entries {
        filter_and_sort(partial, entries.iter().map(|x| &x.name), 0)
    } else {
        Vec::new()
    }
}

pub async fn autocomplete_owned_character_name<'a>(
    ctx: PoiseContext<'a>,
    partial: &'a str,
) -> Vec<String> {
    let guild_id = ctx.guild_id().expect("Command should be guild_only!").get();
    let author_id = ctx.author().id.get();

    filter_and_sort(
        partial,
        ctx.data()
            .cache
            .get_characters()
            .await
            .iter()
            .filter(|(_, cache_item)| cache_item.user_id == author_id)
            .filter(|(_, cache_item)| cache_item.guild_id == guild_id)
            .filter(|(_, cache_item)| !cache_item.is_retired)
            .map(|(_, cache_item)| cache_item.get_autocomplete_name()),
        0,
    )
}

pub async fn autocomplete_retired_character_name<'a>(
    ctx: PoiseContext<'a>,
    partial: &'a str,
) -> Vec<String> {
    let guild_id = ctx.guild_id().expect("Command should be guild_only!").get();

    filter_and_sort(
        partial,
        ctx.data()
            .cache
            .get_characters()
            .await
            .iter()
            .filter(|(_, cache_item)| cache_item.guild_id == guild_id)
            .filter(|(_, cache_item)| cache_item.is_retired)
            .map(|(_, cache_item)| cache_item.get_autocomplete_name()),
        0,
    )
}
