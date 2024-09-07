use crate::commands::Context;
use crate::enums::PokemonTypeWithoutShadow;
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

pub async fn autocomplete_move<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    filter_and_sort(partial, ctx.data().game.move_names.iter(), 2)
}

pub async fn autocomplete_ability<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    filter_and_sort(partial, ctx.data().game.ability_names.iter(), 2)
}

pub async fn autocomplete_pokemon<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    filter_and_sort(partial, ctx.data().game.pokemon_names.iter(), 2)
}

pub async fn autocomplete_item<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    filter_and_sort(partial, ctx.data().game.item_names.iter(), 2)
}

pub async fn autocomplete_weather<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    filter_and_sort(partial, ctx.data().game.weather_names.iter(), 0)
}

pub async fn autocomplete_status_effect<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    filter_and_sort(partial, ctx.data().game.status_effects_names.iter(), 0)
}

pub async fn autocomplete_rule<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
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

pub async fn autocomplete_nature<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    filter_and_sort(partial, ctx.data().game.nature_names.iter(), 0)
}

pub async fn autocomplete_potion<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    filter_and_sort(partial, ctx.data().game.potion_names.iter(), 0)
}

pub async fn autocomplete_pokemon_type<'a>(_ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    filter_and_sort(partial, PokemonTypeWithoutShadow::get_names_vec().iter(), 0)
}

pub async fn autocomplete_character_name<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    filter_and_sort(
        partial,
        ctx.data()
            .cache
            .get_characters()
            .await
            .iter()
            .filter(|x| x.guild_id == ctx.guild_id().expect("Command should be guild_only!").get())
            .filter(|x| !x.is_retired)
            .map(|x| x.get_autocomplete_name()),
        0,
    )
}

pub async fn autocomplete_wallet_name<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
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
    ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> {
    filter_and_sort(
        partial,
        ctx.data()
            .cache
            .get_characters()
            .await
            .iter()
            .filter(|x| x.user_id == ctx.author().id.get())
            .filter(|x| x.guild_id == ctx.guild_id().expect("Command should be guild_only!").get())
            .filter(|x| !x.is_retired)
            .map(|x| x.get_autocomplete_name()),
        0,
    )
}

pub async fn autocomplete_retired_character_name<'a>(
    ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> {
    filter_and_sort(
        partial,
        ctx.data()
            .cache
            .get_characters()
            .await
            .iter()
            .filter(|x| x.guild_id == ctx.guild_id().expect("Command should be guild_only!").get())
            .filter(|x| x.is_retired)
            .map(|x| x.get_autocomplete_name()),
        0,
    )
}
