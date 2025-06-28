use crate::shared::game_data::pokemon::Pokemon;
use crate::shared::game_data::{GameData, PokemonApiId};

pub fn calculate_available_combat_points(level: i64) -> i64 {
    level + 3
}

const STAGE1_EVOLUTION_LEVEL_THRESHOLD: i64 = 3;
const STAGE2_EVOLUTION_LEVEL_THRESHOLD: i64 = 6;

pub fn get_usual_evolution_stage_for_level<'a>(
    level: i64,
    pokemon: &'a Pokemon,
    game_data: &'a GameData,
    stat_override: Option<i64>,
) -> &'a Pokemon {
    if let Some(stat_override) = stat_override {
        let api_id = PokemonApiId(stat_override as u16);
        return game_data.pokemon_by_api_id.get(&api_id).unwrap();
    }

    if pokemon.evolves_from.is_none() {
        return pokemon;
    }
    let evolves_from = pokemon.evolves_from.unwrap();

    if level >= STAGE2_EVOLUTION_LEVEL_THRESHOLD {
        return pokemon;
    }

    let pre_evolution = game_data
        .pokemon_by_api_id
        .get(&evolves_from)
        .expect("Pre-Evolutions should be implemented!");

    if pre_evolution.evolves_from.is_none() {
        // Confirmed one stage evo
        return if level >= STAGE1_EVOLUTION_LEVEL_THRESHOLD {
            pokemon
        } else {
            pre_evolution
        };
    }

    // Confirmed two stage evo
    if level >= STAGE1_EVOLUTION_LEVEL_THRESHOLD {
        return pre_evolution;
    }

    // Confirmed first stage stats
    game_data
        .pokemon_by_api_id
        .get(&pre_evolution.evolves_from.unwrap())
        .expect("Pre-Evolutions should be implemented!")
}

pub fn calculate_level_from_experience(experience: i64) -> i64 {
    experience / 100 + 1
}

pub fn calculate_current_experience(experience: i64) -> i64 {
    experience % 100
}

pub fn calculate_next_limit_break_cost(limit_break_count: i64) -> i64 {
    2 + limit_break_count
}
