use std::cmp::Ordering;
use std::sync::Arc;
use crate::commands::Context;

fn autocomplete(partial: &str, commands: &Arc<Vec<String>>, minimum_query_length : usize) -> Vec<String> {
    if partial.len() < minimum_query_length {
        return Vec::default();
    }

    let lower_case = &partial.to_lowercase();

    let mut result: Vec<String> = commands.iter()
        .filter(move |x| x.to_lowercase().contains(lower_case))
        .cloned()
        .collect();

    result.sort_by(|a, b| {
        if a.to_lowercase().starts_with(lower_case) {
            if b.to_lowercase().starts_with(lower_case) {
                return a.cmp(b);
            }
            return Ordering::Less;
        }
        if b.to_lowercase().starts_with(lower_case) {
            return Ordering::Greater;
        }

        Ordering::Equal
    });

    result
}

pub async fn autocomplete_move<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> {
    autocomplete(partial, &_ctx.data().game.move_names, 2)
}

pub async fn autocomplete_ability<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> {
    autocomplete(partial, &_ctx.data().game.ability_names, 2)
}

pub async fn autocomplete_pokemon<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> {
    autocomplete(partial, &_ctx.data().game.pokemon_names, 2)
}

pub async fn autocomplete_item<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> { autocomplete(partial, &_ctx.data().game.item_names, 2) }

pub async fn autocomplete_weather<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> { autocomplete(partial, &_ctx.data().game.weather_names, 0) }

pub async fn autocomplete_status_effect<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> { autocomplete(partial, &_ctx.data().game.status_effects_names, 0) }

pub async fn autocomplete_rule<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> { autocomplete(partial, &_ctx.data().game.rule_names, 0) }

pub async fn autocomplete_nature<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> { autocomplete(partial, &_ctx.data().game.nature_names, 0) }

pub async fn autocomplete_potion<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> { autocomplete(partial, &_ctx.data().game.potion_names, 0) }

pub async fn autocomplete_character_name<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<String> {
    autocomplete(partial, &_ctx.data().cache.lock().await.character_names, 0)
}
