use std::collections::{HashMap};
use log::error;
use strum::IntoEnumIterator;
use crate::csv_utils::load_csv;
use crate::game_data::pokemon::{Height, Weight};
use crate::game_data::type_efficiency::TypeEfficiency;
use crate::enums::{PokemonGeneration, PokemonType};
use crate::game_data::pokemon_api::api_types::*;
use crate::game_data::pokemon_api::PokemonApiId;

#[derive(Debug)]
pub struct ApiMoveEntry {
    pub move_name: String,
    pub generation: PokemonGeneration,
}

#[derive(Debug)]
pub struct ApiPokemonLearnableMoves {
    pub level_up: Vec<ApiMoveEntry>,
    pub machine: Vec<ApiMoveEntry>,
    pub tutor: Vec<ApiMoveEntry>,
    pub egg: Vec<ApiMoveEntry>,
}

#[derive(Debug)]
pub struct PokemonApiData {
    pub pokemon_id: PokemonApiId,
    pub pokemon_name: String,
    pub generation: PokemonGeneration,
    pub has_gender_differences: bool,
    pub height: Height,
    pub weight: Weight,
    pub type1: PokemonType,
    pub type2: Option<PokemonType>,
    pub ability1: String,
    pub ability2: Option<String>,
    pub ability_hidden: Option<String>,
    pub ability_event: Option<String>,
    pub learnable_moves: ApiPokemonLearnableMoves,
}

impl ApiPokemonLearnableMoves {
    fn has_move(&self, name: String, learn_method: &str) -> bool {
        match learn_method {
            "level-up" => self.level_up.iter().any(|x| x.move_name == name),
            "egg" => self.egg.iter().any(|x| x.move_name == name),
            "tutor" => self.tutor.iter().any(|x| x.move_name == name),
            "machine" => self.machine.iter().any(|x| x.move_name == name),
            _ => false
        }
    }
}

fn type_id_to_pokemon_type(id: TypeId) -> PokemonType {
    match id.0 {
        1 => PokemonType::Normal,
        2 => PokemonType::Fighting,
        3 => PokemonType::Flying,
        4 => PokemonType::Poison,
        5 => PokemonType::Ground,
        6 => PokemonType::Rock,
        7 => PokemonType::Bug,
        8 => PokemonType::Ghost,
        9 => PokemonType::Steel,
        10 => PokemonType::Fire,
        11 => PokemonType::Water,
        12 => PokemonType::Grass,
        13 => PokemonType::Electric,
        14 => PokemonType::Psychic,
        15 => PokemonType::Ice,
        16 => PokemonType::Dragon,
        17 => PokemonType::Dark,
        18 => PokemonType::Fairy,
        10001 => PokemonType::Normal, // Unknown but ... what pokemon has unknown type?! D:
        10002 => PokemonType::Shadow,
        _ => panic!("Weird type id: {}", id.0)
    }
}

fn generation_id_to_generation(id: &GenerationId) -> PokemonGeneration {
    match id.0 {
        1 => PokemonGeneration::One,
        2 => PokemonGeneration::Two,
        3 => PokemonGeneration::Three,
        4 => PokemonGeneration::Four,
        5 => PokemonGeneration::Five,
        6 => PokemonGeneration::Six,
        7 => PokemonGeneration::Seven,
        8 => PokemonGeneration::Eight,
        9 => PokemonGeneration::Nine,
        _ => panic!("Weird generation id: {}", id.0)
    }
}

pub fn parse_type_efficacy(path: String) -> TypeEfficiency {
    let csv: Vec<ApiTypeEfficacy> = load_csv(path + "data/v2/csv/type_efficacy.csv");

    let mut result: HashMap<PokemonType, HashMap<PokemonType, f32>> = HashMap::default();
    for x in csv {
        let this_type = type_id_to_pokemon_type(x.damage_type_id);
        let entry_option = result.get_mut(&this_type);
        let entry = match entry_option {
            Some(e) => e,
            None => {
                result.insert(this_type, HashMap::default());
                result.get_mut(&this_type).unwrap()
            }
        };

        let target_type = type_id_to_pokemon_type(x.target_type_id);
        entry.insert(target_type, x.damage_factor as f32 * 0.01);
    }

    // FIXME: Technically this has nothing to do with api parsing anymore... :D
    let mut shadow: HashMap<PokemonType, f32> = HashMap::default();
    for x in PokemonType::iter() {
        shadow.insert(x, 2.0);
    }
    result.insert(PokemonType::Shadow, shadow);
    for x in PokemonType::iter() {
        result.get_mut(&x).unwrap().insert(PokemonType::Shadow, 2.0);
    }

    TypeEfficiency::new(result)
}

pub fn parse_pokemon_api(path: String) -> HashMap<String, PokemonApiData> {
    let english_language_id:u8 = 9;
    let version_groups: Vec<ApiVersionGroups> = load_csv(path.clone() + "data/v2/csv/version_groups.csv");
    let ability_names: Vec<ApiAbilityName> = load_csv(path.clone() + "data/v2/csv/ability_names.csv");
    let pokemon: Vec<ApiPokemon> = load_csv(path.clone() + "data/v2/csv/pokemon.csv");
    let pokemon_abilities: Vec<ApiPokemonAbility> = load_csv(path.clone() + "data/v2/csv/pokemon_abilities.csv");
    let pokemon_types: Vec<ApiPokemonTypes> = load_csv(path.clone() + "data/v2/csv/pokemon_types.csv");
    let pokemon_moves: Vec<ApiPokemonMoves> = load_csv(path.clone() + "data/v2/csv/pokemon_moves.csv");
    let pokemon_move_methods: Vec<ApiPokemonMoveMethods> = load_csv(path.clone() + "data/v2/csv/pokemon_move_methods.csv");
    let pokemon_species: Vec<ApiPokemonSpecies> = load_csv(path.clone() + "data/v2/csv/pokemon_species.csv");
    let pokemon_species_names: Vec<ApiPokemonSpeciesNames> = load_csv(path.clone() + "data/v2/csv/pokemon_species_names.csv");
    let pokemon_forms: Vec<ApiPokemonForm> = load_csv(path.clone() + "data/v2/csv/pokemon_forms.csv");
    let pokemon_form_types: Vec<ApiPokemonFormTypes> = load_csv(path.clone() + "data/v2/csv/pokemon_form_types.csv");
    let pokemon_form_names: Vec<ApiPokemonFormNames> = load_csv(path.clone() + "data/v2/csv/pokemon_form_names.csv");
    let move_names: Vec<ApiMoveNames> = load_csv(path + "data/v2/csv/move_names.csv");

    let mut ability_id_to_name: HashMap<AbilityId, String> = HashMap::default();
    for x in ability_names {
        if x.local_language_id != english_language_id {
            continue;
        }

        ability_id_to_name.insert(x.ability_id, x.name);
    }

    let mut form_id_to_pokemon_id: HashMap<PokemonFormId, PokemonApiId> = HashMap::default();
    for x in pokemon_forms {
        form_id_to_pokemon_id.insert(x.id, x.pokemon_id);
    }

    let mut pokemon_id_to_pokemon_ability1: HashMap<PokemonApiId, String> = HashMap::default();
    let mut pokemon_id_to_pokemon_ability2: HashMap<PokemonApiId, String> = HashMap::default();
    let mut pokemon_id_to_pokemon_ability_hidden: HashMap<PokemonApiId, String> = HashMap::default();
    for x in pokemon_abilities {
        match x.slot {
            1 => pokemon_id_to_pokemon_ability1.insert(x.pokemon_id, ability_id_to_name.get(&x.ability_id).expect("Ability should be set!").clone()),
            2 => pokemon_id_to_pokemon_ability2.insert(x.pokemon_id, ability_id_to_name.get(&x.ability_id).expect("Ability should be set!").clone()),
            3 => pokemon_id_to_pokemon_ability_hidden.insert(x.pokemon_id, ability_id_to_name.get(&x.ability_id).expect("Ability should be set!").clone()),
            _ => None,
        };
    }

    let mut pokemon_id_to_pokemon_type1: HashMap<PokemonApiId, PokemonType> = HashMap::default();
    let mut pokemon_id_to_pokemon_type2: HashMap<PokemonApiId, PokemonType> = HashMap::default();
    for x in pokemon_types {
        if x.slot == 1 {
            pokemon_id_to_pokemon_type1.insert(x.pokemon_id, type_id_to_pokemon_type(x.type_id));
        } else {
            pokemon_id_to_pokemon_type2.insert(x.pokemon_id, type_id_to_pokemon_type(x.type_id));
        }
    }
    for x in pokemon_form_types {
        if let Some(pokemon_id) = form_id_to_pokemon_id.get(&x.pokemon_form_id) {
            if x.slot == 1 {
                pokemon_id_to_pokemon_type1.insert(PokemonApiId(pokemon_id.0), type_id_to_pokemon_type(x.type_id));
            } else {
                pokemon_id_to_pokemon_type2.insert(PokemonApiId(pokemon_id.0), type_id_to_pokemon_type(x.type_id));
            }
        } else {
            error!("Unable to map pokemon form id {} to a pokemon id!", x.pokemon_form_id.0);
        }
    }

    let mut pokemon_id_to_name: HashMap<PokemonApiId, String> = HashMap::default();
    for x in pokemon_species_names {
        if x.local_language_id != english_language_id {
            continue;
        }

        // These should be always the same for species < 10000
        pokemon_id_to_name.insert(PokemonApiId(x.pokemon_species_id.0), x.name);
    }
    for x in pokemon_form_names {
        if x.local_language_id != english_language_id {
            continue;
        }

        if let Some(name) = x.pokemon_name {
            if let Some(pokemon_id) = form_id_to_pokemon_id.get(&x.pokemon_form_id) {
                pokemon_id_to_name.insert(PokemonApiId(pokemon_id.0), name);
            } else {
                error!("Unable to map pokemon form id {} to a pokemon id!", x.pokemon_form_id.0);
            }
        }
    }

    let mut species_id_to_species: HashMap<PokemonSpeciesId, ApiPokemonSpecies> = HashMap::default();
    for x in pokemon_species {
        species_id_to_species.insert(x.id, x);
    }

    let mut move_id_to_name: HashMap<MoveId, String> = HashMap::default();
    for x in move_names {
        if x.local_language_id != english_language_id {
            continue;
        }

        move_id_to_name.insert(x.move_id, x.name);
    }

    let mut method_id_to_name: HashMap<u8, String> = HashMap::default();
    for x in pokemon_move_methods {
        method_id_to_name.insert(x.id, x.identifier);
    }

    let mut version_group_id_to_generation: HashMap<u8, PokemonGeneration> = HashMap::default();
    for x in version_groups {
        version_group_id_to_generation.insert(x.id, generation_id_to_generation(&x.generation_id));
    }

    let mut result: HashMap<String, PokemonApiData> = HashMap::default();
    for x in pokemon {
        let name = pokemon_id_to_name.get(&x.id).unwrap_or(&x.identifier);
        let species = species_id_to_species.get(&x.species_id)
            .unwrap_or_else(|| panic!("Species should always be available, but was not for {}", name));

        result.insert(name.clone(),  PokemonApiData {
            pokemon_id: PokemonApiId(x.id.0),
            pokemon_name: name.clone(),
            generation: generation_id_to_generation(&species.generation_id),
            has_gender_differences: species.has_gender_differences > 0,
            height: Height {meters: x.height as f32 / 10.0, feet: x.height as f32 / 10.0 * 3.28084 },
            weight: Weight {kilograms: x.weight as f32 / 10.0, pounds: x.weight as f32 / 10.0 * 2.20462 },
            type1: pokemon_id_to_pokemon_type1.get(&x.id).expect("").to_owned(),
            type2: pokemon_id_to_pokemon_type2.get(&x.id).copied(),
            ability1: pokemon_id_to_pokemon_ability1.get(&x.id).expect("").clone(),
            ability2: pokemon_id_to_pokemon_ability2.get(&x.id).cloned(),
            ability_hidden: pokemon_id_to_pokemon_ability_hidden.get(&x.id).cloned(),
            ability_event: None,
            learnable_moves: ApiPokemonLearnableMoves {
                level_up: Vec::default(),
                machine: Vec::default(),
                tutor: Vec::default(),
                egg: Vec::default(),
            },
        });
    }

    let mut missing_pokemon_ids = Vec::new();
    let mut missing_move_ids = Vec::new();
    for pokemon_move in pokemon_moves {
        if let Some(pokemon_name) = pokemon_id_to_name.get(&pokemon_move.pokemon_id) {
            let move_name_option = move_id_to_name.get(&pokemon_move.move_id);
            if move_name_option.is_none() {
                if !missing_move_ids.contains(&pokemon_move.move_id) {
                    missing_move_ids.push(pokemon_move.move_id);
                }
                continue;
            }
            let move_name = move_name_option.unwrap().clone();

            let pokemon_entry = &mut result.get_mut(pokemon_name).unwrap().learnable_moves;
            let learn_method = method_id_to_name.get(&pokemon_move.pokemon_move_method_id).unwrap().clone();
            if pokemon_entry.has_move(move_name.clone(), &learn_method) {
                continue;
            }

            let new_move_entry = ApiMoveEntry {
                move_name: move_name.clone(),
                generation: *version_group_id_to_generation.get(&pokemon_move.version_group_id).expect("All generation ids should be set"),
            };

            match learn_method.as_str() {
                "level-up" => pokemon_entry.level_up.push(new_move_entry),
                "egg" => pokemon_entry.egg.push(new_move_entry),
                "tutor" => pokemon_entry.tutor.push(new_move_entry),
                "machine" => pokemon_entry.machine.push(new_move_entry),
                _ => {}
            }
        } else if !missing_pokemon_ids.contains(&pokemon_move.pokemon_id) {
            missing_pokemon_ids.push(pokemon_move.pokemon_id);
        }
    }

    for x in missing_pokemon_ids {
        // 10250 - 10271 is Amigento, skip that one for now
        if !(10250..=10271).contains(&x.0) {
            log::warn!("Missing pokemon data for pokemon_id {}", x.0)
        }

    }
    for x in missing_move_ids {
        log::warn!("Missing move data for move_id {}", x.0)
    }


    result
}
