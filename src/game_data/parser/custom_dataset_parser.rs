use crate::game_data::ability::Ability;
use crate::game_data::item::Item;
use crate::game_data::parser::custom_data::parser::CustomDataParseResult;
use crate::game_data::parser::{custom_data, helpers};
use crate::game_data::pokemon::Pokemon;
use crate::game_data::pokemon_api::pokemon_api_parser::PokemonApiData;
use crate::game_data::potion::Potion;
use crate::game_data::r#move::Move;
use crate::game_data::status_effect::StatusEffect;
use crate::game_data::weather::Weather;
use crate::game_data::GameData;
use log::{error, info};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct CustomDataSet {
    name: String,
    path: String,
    server_id: i64,
    fallback_id: Option<i64>,
}

pub fn parse(
    base_path: String,
    base_data: &GameData,
    pokemon_api_data: &HashMap<String, PokemonApiData>,
) -> HashMap<i64, GameData> {
    let custom_data_sets = helpers::parse_file::<Vec<CustomDataSet>>(&format!(
        "{base_path}custom_server_data/data_mapping.json"
    ))
    .expect("This file should always exist!");

    let mut result = HashMap::default();
    // TODO: Parse in order of fallback_id, allowing datasets to "depend" upon each other
    for x in custom_data_sets {
        info!("Parsing custom data set: {}", x.path);
        let parsed_data = custom_data::parser::parse(&format!("{base_path}{}/", x.path));
        let parsed_data_set = parse_custom(
            base_data,
            x.server_id,
            x.name,
            parsed_data,
            pokemon_api_data,
        );
        result.insert(x.server_id, parsed_data_set);
    }

    result
}

pub fn parse_custom(
    base_data: &GameData,
    id: i64,
    name: String,
    custom: CustomDataParseResult,
    pokemon_api_data: &HashMap<String, PokemonApiData>,
) -> GameData {
    let mut data = base_data.clone();

    data.name = name;
    data.id = id;

    for x in custom.pokemon {
        if let Some(pokemon) = Pokemon::from_custom_data(&x, pokemon_api_data) {
            if data.pokemon.insert(x.name.clone(), pokemon).is_none() {
                data.pokemon_names.push(x.name)
            };
        } else {
            error!(
                "Was unable to parse custom pokemon {} in dataset {}",
                x.name, data.name
            );

            // TODO: Log that into the bot info channel for the respective server
        }
    }

    add_custom_data(
        custom.abilities,
        &mut data.abilities,
        &mut data.ability_names,
        Ability::from_custom_data,
        |x| x.name.clone(),
    );

    add_custom_data(
        custom.items,
        &mut data.items,
        &mut data.item_names,
        Item::from_custom_data,
        |x| x.name.clone(),
    );

    add_custom_data(
        custom.moves,
        &mut data.moves,
        &mut data.move_names,
        Move::from_custom_data,
        |x| x.name.clone(),
    );

    add_custom_data(
        custom.potions,
        &mut data.potions,
        &mut data.potion_names,
        Potion::from_custom_data,
        |x| x.name.clone(),
    );

    add_custom_data(
        custom.status_effects,
        &mut data.status_effects,
        &mut data.status_effects_names,
        StatusEffect::from_custom_data,
        |x| x.name.clone(),
    );

    add_custom_data(
        custom.weather,
        &mut data.weather,
        &mut data.weather_names,
        Weather::from_custom_data,
        |x| x.name.clone(),
    );

    data
}

fn add_custom_data<TInput, TOutput, FnCreate, FnName>(
    data_to_add: Vec<TInput>,
    collection: &mut HashMap<String, TOutput>,
    item_names: &mut Vec<String>,
    create_fn: FnCreate,
    name_fn: FnName,
) where
    FnCreate: Fn(&TInput) -> TOutput,
    FnName: Fn(&TInput) -> String,
{
    for x in data_to_add {
        let name = name_fn(&x);
        if collection.insert(name.clone(), create_fn(&x)).is_none() {
            item_names.push(name)
        };
    }
}
