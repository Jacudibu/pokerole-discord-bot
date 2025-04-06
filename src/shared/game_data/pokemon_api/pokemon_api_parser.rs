use crate::shared::enums::{PokemonGeneration, PokemonType};
use crate::shared::game_data::pokemon::{Height, Weight};
use crate::shared::game_data::pokemon_api::api_types::*;
use crate::shared::game_data::pokemon_api::PokemonApiId;
use crate::shared::game_data::type_efficiency::TypeEfficiency;
use log::{error, warn};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::Path;
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct ApiMoveEntry {
    pub move_name: String,
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
    pub evolves_from: Option<PokemonApiId>,
    pub has_gender_differences: bool,
    pub height: Height,
    pub weight: Weight,
    pub type1: PokemonType,
    pub type2: Option<PokemonType>,
    pub abilities: ApiPokemonAbilities,
    pub learnable_moves: ApiPokemonLearnableMoves,
    pub pokedex_entries: Vec<PokedexEntry>,
}

impl ApiPokemonLearnableMoves {
    fn has_move(&self, name: &str, learn_method: &MoveLearnMethodId) -> bool {
        match *learn_method {
            LEVEL_UP => self.level_up.iter().any(|x| x.move_name == name),
            EGG => self.egg.iter().any(|x| x.move_name == name),
            TUTOR => self.tutor.iter().any(|x| x.move_name == name),
            MACHINE => self.machine.iter().any(|x| x.move_name == name),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PokedexEntry {
    pub version: String,
    pub text: String,
}

impl PokedexEntry {
    pub fn new(version: String, text: String) -> Self {
        PokedexEntry {
            version,
            text: text.replace('\n', " "),
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
        _ => panic!("Weird type id: {}", id.0),
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
        _ => panic!("Weird generation id: {}", id.0),
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

const ENGLISH_LANGUAGE_ID: LanguageId = LanguageId(9);

#[derive(Debug)]
pub struct ApiPokemonAbilities {
    pub ability1: String,
    pub ability2: Option<String>,
    pub hidden: Option<String>,
    pub event: Option<String>,
}

impl ApiPokemonAbilities {
    fn new(ability1: String) -> Self {
        ApiPokemonAbilities {
            ability1,
            ability2: None,
            hidden: None,
            event: None,
        }
    }
}

pub fn parse_pokemon_api(path: String) -> HashMap<String, PokemonApiData> {
    // let version_groups: Vec<ApiVersionGroups> = load_csv(path.clone() + "data/v2/csv/version_groups.csv");
    let ability_names: Vec<ApiAbilityName> =
        load_csv(path.clone() + "data/v2/csv/ability_names.csv");
    let pokemon: Vec<ApiPokemon> = load_csv(path.clone() + "data/v2/csv/pokemon.csv");
    let pokemon_abilities: Vec<ApiPokemonAbility> =
        load_csv(path.clone() + "data/v2/csv/pokemon_abilities.csv");
    let pokemon_types: Vec<ApiPokemonTypes> =
        load_csv(path.clone() + "data/v2/csv/pokemon_types.csv");
    let pokemon_moves: Vec<ApiPokemonMoves> =
        load_csv(path.clone() + "data/v2/csv/pokemon_moves.csv");
    // let pokemon_move_methods: Vec<ApiPokemonMoveMethods> = load_csv(path.clone() + "data/v2/csv/pokemon_move_methods.csv");
    let pokemon_species: Vec<ApiPokemonSpecies> =
        load_csv(path.clone() + "data/v2/csv/pokemon_species.csv");
    let pokemon_species_names: Vec<ApiPokemonSpeciesNames> =
        load_csv(path.clone() + "data/v2/csv/pokemon_species_names.csv");
    let pokemon_species_flavor_text: Vec<ApiPokemonSpeciesFlavorText> =
        load_csv(path.clone() + "data/v2/csv/pokemon_species_flavor_text.csv");
    let pokemon_forms: Vec<ApiPokemonForm> =
        load_csv(path.clone() + "data/v2/csv/pokemon_forms.csv");
    let pokemon_form_types: Vec<ApiPokemonFormTypes> =
        load_csv(path.clone() + "data/v2/csv/pokemon_form_types.csv");
    let pokemon_form_names: Vec<ApiPokemonFormNames> =
        load_csv(path.clone() + "data/v2/csv/pokemon_form_names.csv");
    let move_names: Vec<ApiMoveNames> = load_csv(path.clone() + "data/v2/csv/move_names.csv");
    let version_names: Vec<ApiVersionNames> = load_csv(path + "data/v2/csv/version_names.csv");

    let ability_id_to_name = map_ability_id_to_names(ability_names);
    let version_id_to_name = map_version_id_to_name(version_names);
    let species_id_to_flavor_texts =
        map_species_id_to_flavor_texts(pokemon_species_flavor_text, version_id_to_name);
    let (form_id_to_pokemon_form, pokemon_id_to_form_ids) =
        map_form_id_and_pokemon_id(pokemon_forms);
    let mut pokemon_id_to_abilities =
        map_pokemon_id_to_abilities(pokemon_abilities, ability_id_to_name);
    let (pokemon_id_to_pokemon_type1, pokemon_id_to_pokemon_type2) =
        map_pokemon_id_to_types(pokemon_types, pokemon_form_types, &form_id_to_pokemon_form);
    let pokemon_id_to_name = map_pokemon_id_to_name(
        pokemon_species_names,
        pokemon_form_names,
        &form_id_to_pokemon_form,
    );
    let species_id_to_pokemon_ids = map_species_id_to_pokemon_ids(&pokemon);
    let species_id_to_species = map_species_id_to_species(pokemon_species);
    let move_id_to_name = map_move_id_to_name(move_names);
    // let move_learn_method_id_to_name = map_move_learn_method_id_to_name(pokemon_move_methods);
    // let version_group_id_to_generation = map_version_group_id_to_generation(version_groups);
    let mut pokemon_id_to_moves = map_pokemon_id_to_moves(pokemon_moves);

    let mut result: HashMap<String, PokemonApiData> = HashMap::default();
    for x in pokemon {
        let name = pokemon_id_to_name.get(&x.id).unwrap_or(&x.identifier);
        let species = species_id_to_species.get(&x.species_id).unwrap_or_else(|| {
            panic!(
                "Species should always be available, but was not for {}",
                name
            )
        });

        let abilities = pokemon_id_to_abilities.remove(&x.id).unwrap_or_else(|| {
            panic!(
                "Pokemon should always have abilities, but none found for {}",
                name
            )
        });

        let moves = pokemon_id_to_moves.remove(&x.id).unwrap_or_else(|| {
            if !name.starts_with("Gigantamax") {
                // Those just don't have moves right now. But we aren't using them anyway, so ye...
                warn!(
                    "Pokemon should always have moves, but none found for {}",
                    name
                );
            }
            Vec::new()
        });

        result.insert(
            name.clone(),
            PokemonApiData {
                pokemon_id: PokemonApiId(x.id.0),
                pokemon_name: name.clone(),
                generation: generation_id_to_generation(&species.generation_id),
                evolves_from: get_evo_origin(
                    &x.id,
                    &x.species_id,
                    &species_id_to_species,
                    &species_id_to_pokemon_ids,
                    &pokemon_id_to_form_ids,
                    &form_id_to_pokemon_form,
                ),
                has_gender_differences: species.has_gender_differences > 0,
                height: Height {
                    meters: x.height as f32 / 10.0,
                    feet: x.height as f32 / 10.0 * 3.28084,
                },
                weight: Weight {
                    kilograms: x.weight as f32 / 10.0,
                    pounds: x.weight as f32 / 10.0 * 2.20462,
                },
                type1: pokemon_id_to_pokemon_type1.get(&x.id).expect("").to_owned(),
                type2: pokemon_id_to_pokemon_type2.get(&x.id).copied(),
                abilities,
                pokedex_entries: species_id_to_flavor_texts
                    .get(&species.id)
                    .cloned()
                    .unwrap_or_default(),
                learnable_moves: get_learnable_moves(moves, &move_id_to_name),
            },
        );
    }

    result
}

fn map_species_id_to_pokemon_ids(
    pokemon: &Vec<ApiPokemon>,
) -> HashMap<PokemonSpeciesId, Vec<PokemonApiId>> {
    let mut species_id_to_pokemon_ids: HashMap<PokemonSpeciesId, Vec<PokemonApiId>> =
        HashMap::default();
    for x in pokemon {
        species_id_to_pokemon_ids
            .entry(x.species_id)
            .or_default()
            .push(x.id);
    }
    species_id_to_pokemon_ids
}

fn get_learnable_moves(
    moves: Vec<ApiPokemonMoves>,
    move_id_to_name: &HashMap<MoveId, String>,
) -> ApiPokemonLearnableMoves {
    let mut result = ApiPokemonLearnableMoves {
        level_up: Vec::default(),
        machine: Vec::default(),
        tutor: Vec::default(),
        egg: Vec::default(),
    };

    for x in moves {
        let move_name_option = move_id_to_name.get(&x.move_id);
        let move_name = match move_name_option {
            None => {
                warn!("Missing move name for move_id {}", x.move_id.0);
                continue;
            }
            Some(name) => name,
        };

        if result.has_move(move_name, &x.pokemon_move_method_id) {
            continue;
        }

        let new_move_entry = ApiMoveEntry {
            move_name: move_name.clone(),
        };

        match x.pokemon_move_method_id {
            LEVEL_UP => result.level_up.push(new_move_entry),
            EGG => result.egg.push(new_move_entry),
            TUTOR => result.tutor.push(new_move_entry),
            MACHINE => result.machine.push(new_move_entry),
            _ => {}
        }
    }

    result
}

fn get_evo_origin(
    pokemon_id: &PokemonApiId,
    species_id: &PokemonSpeciesId,
    species_id_to_species: &HashMap<PokemonSpeciesId, ApiPokemonSpecies>,
    species_id_to_pokemon_ids: &HashMap<PokemonSpeciesId, Vec<PokemonApiId>>,
    pokemon_id_to_form_ids: &HashMap<PokemonApiId, Vec<PokemonFormId>>,
    form_id_to_pokemon_form: &HashMap<PokemonFormId, ApiPokemonForm>,
) -> Option<PokemonApiId> {
    let species = species_id_to_species
        .get(species_id)
        .expect("Every mon should have a species!");

    let evolves_from = species.evolves_from_species_id?;
    let base_pokemon = species_id_to_pokemon_ids
        .get(&evolves_from)
        .expect("Every species should have at least one mon attached!");

    if base_pokemon.len() == 1 {
        return base_pokemon.first().cloned();
    }

    let evolved_forms = pokemon_id_to_form_ids
        .get(pokemon_id)
        .expect("Every mon should have at least one form");

    for evolved_form_id in evolved_forms {
        let evolved_form = form_id_to_pokemon_form
            .get(evolved_form_id)
            .expect("Every form id should have a form!");

        if evolved_form.is_default == 0 {
            continue;
        }

        for potential_base_pokemon_id in base_pokemon {
            let base_pokemon_forms = pokemon_id_to_form_ids
                .get(potential_base_pokemon_id)
                .expect("Every mon should have at least one form");

            for base_pokemon_form_id in base_pokemon_forms {
                let base_pokemon_form = form_id_to_pokemon_form
                    .get(base_pokemon_form_id)
                    .expect("Every form id should have a form!");

                if base_pokemon_form.is_default == 0 {
                    continue;
                }

                if base_pokemon_form.form_identifier == evolved_form.form_identifier {
                    return Some(*potential_base_pokemon_id);
                }
            }
        }
    }

    Some(PokemonApiId(evolves_from.0))
}

fn map_pokemon_id_to_moves(
    pokemon_moves: Vec<ApiPokemonMoves>,
) -> HashMap<PokemonApiId, Vec<ApiPokemonMoves>> {
    let mut result: HashMap<PokemonApiId, Vec<ApiPokemonMoves>> = HashMap::default();
    for x in pokemon_moves {
        result.entry(x.pokemon_id).or_default().push(x);
    }

    result
}

fn _map_version_group_id_to_generation(
    version_groups: Vec<ApiVersionGroups>,
) -> HashMap<u8, PokemonGeneration> {
    let mut version_group_id_to_generation: HashMap<u8, PokemonGeneration> = HashMap::default();
    for x in version_groups {
        version_group_id_to_generation.insert(x.id, generation_id_to_generation(&x.generation_id));
    }
    version_group_id_to_generation
}

fn _map_move_learn_method_id_to_name(
    pokemon_move_methods: Vec<ApiPokemonMoveMethods>,
) -> HashMap<MoveLearnMethodId, String> {
    let mut move_learn_method_id_to_name: HashMap<MoveLearnMethodId, String> = HashMap::default();
    for x in pokemon_move_methods {
        move_learn_method_id_to_name.insert(x.id, x.identifier);
    }
    move_learn_method_id_to_name
}

fn map_move_id_to_name(move_names: Vec<ApiMoveNames>) -> HashMap<MoveId, String> {
    let mut move_id_to_name: HashMap<MoveId, String> = HashMap::default();
    for x in move_names {
        if x.local_language_id != ENGLISH_LANGUAGE_ID {
            continue;
        }

        move_id_to_name.insert(x.move_id, x.name);
    }
    move_id_to_name
}

fn map_species_id_to_species(
    pokemon_species: Vec<ApiPokemonSpecies>,
) -> HashMap<PokemonSpeciesId, ApiPokemonSpecies> {
    let mut species_id_to_species: HashMap<PokemonSpeciesId, ApiPokemonSpecies> =
        HashMap::default();
    for x in pokemon_species {
        species_id_to_species.insert(x.id, x);
    }
    species_id_to_species
}

fn map_pokemon_id_to_name(
    pokemon_species_names: Vec<ApiPokemonSpeciesNames>,
    pokemon_form_names: Vec<ApiPokemonFormNames>,
    form_id_to_pokemon_id: &HashMap<PokemonFormId, ApiPokemonForm>,
) -> HashMap<PokemonApiId, String> {
    let mut pokemon_id_to_name: HashMap<PokemonApiId, String> = HashMap::default();
    for x in pokemon_species_names {
        if x.local_language_id != ENGLISH_LANGUAGE_ID {
            continue;
        }

        // These should be always the same for species < 10000
        pokemon_id_to_name.insert(PokemonApiId(x.pokemon_species_id.0), x.name);
    }
    for x in pokemon_form_names {
        if x.local_language_id != ENGLISH_LANGUAGE_ID {
            continue;
        }

        if let Some(name) = x.pokemon_name {
            if let Some(form) = form_id_to_pokemon_id.get(&x.pokemon_form_id) {
                pokemon_id_to_name.insert(form.pokemon_id, name);
            } else {
                error!(
                    "Unable to map pokemon form id {} to a pokemon id!",
                    x.pokemon_form_id.0
                );
            }
        }
    }
    pokemon_id_to_name
}

fn map_pokemon_id_to_types(
    pokemon_types: Vec<ApiPokemonTypes>,
    pokemon_form_types: Vec<ApiPokemonFormTypes>,
    form_id_to_form: &HashMap<PokemonFormId, ApiPokemonForm>,
) -> (
    HashMap<PokemonApiId, PokemonType>,
    HashMap<PokemonApiId, PokemonType>,
) {
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
        if let Some(pokemon_form) = form_id_to_form.get(&x.pokemon_form_id) {
            if x.slot == 1 {
                pokemon_id_to_pokemon_type1
                    .insert(pokemon_form.pokemon_id, type_id_to_pokemon_type(x.type_id));
            } else {
                pokemon_id_to_pokemon_type2
                    .insert(pokemon_form.pokemon_id, type_id_to_pokemon_type(x.type_id));
            }
        } else {
            error!(
                "Unable to map pokemon form id {} to a pokemon id!",
                x.pokemon_form_id.0
            );
        }
    }
    (pokemon_id_to_pokemon_type1, pokemon_id_to_pokemon_type2)
}

fn map_pokemon_id_to_abilities(
    pokemon_abilities: Vec<ApiPokemonAbility>,
    ability_id_to_name: HashMap<AbilityId, String>,
) -> HashMap<PokemonApiId, ApiPokemonAbilities> {
    let mut result: HashMap<PokemonApiId, ApiPokemonAbilities> = HashMap::default();
    for x in pokemon_abilities {
        match x.slot {
            1 => {
                result.insert(
                    x.pokemon_id,
                    ApiPokemonAbilities::new(
                        ability_id_to_name
                            .get(&x.ability_id)
                            .expect("Ability should be set!")
                            .clone(),
                    ),
                );
            }
            2 => {
                let abilities = result
                    .get_mut(&x.pokemon_id)
                    .expect("Ability 1 should already have been set!");
                abilities.ability2 = Some(
                    ability_id_to_name
                        .get(&x.ability_id)
                        .expect("Ability should be set!")
                        .clone(),
                );
            }
            3 => {
                let abilities = result
                    .get_mut(&x.pokemon_id)
                    .expect("Ability 1 should already have been set!");
                abilities.hidden = Some(
                    ability_id_to_name
                        .get(&x.ability_id)
                        .expect("Ability should be set!")
                        .clone(),
                );
            }
            _ => {}
        };
    }
    result
}

fn map_form_id_and_pokemon_id(
    pokemon_forms: Vec<ApiPokemonForm>,
) -> (
    HashMap<PokemonFormId, ApiPokemonForm>,
    HashMap<PokemonApiId, Vec<PokemonFormId>>,
) {
    let mut form_id_to_form: HashMap<PokemonFormId, ApiPokemonForm> = HashMap::default();
    let mut pokemon_id_to_form_id: HashMap<PokemonApiId, Vec<PokemonFormId>> = HashMap::default();
    for x in pokemon_forms {
        pokemon_id_to_form_id
            .entry(x.pokemon_id)
            .or_default()
            .push(x.id); // Forms is a superset of Pokemon. Maybe we should make parsing based off of forms?
        form_id_to_form.insert(x.id, x);
    }
    (form_id_to_form, pokemon_id_to_form_id)
}

fn map_species_id_to_flavor_texts(
    pokemon_species_flavor_text: Vec<ApiPokemonSpeciesFlavorText>,
    version_id_to_name: HashMap<VersionId, String>,
) -> HashMap<PokemonSpeciesId, Vec<PokedexEntry>> {
    let mut species_id_to_flavor_texts: HashMap<PokemonSpeciesId, Vec<PokedexEntry>> =
        HashMap::default();
    for x in pokemon_species_flavor_text {
        if x.language_id != ENGLISH_LANGUAGE_ID {
            continue;
        }

        let version_name = version_id_to_name
            .get(&x.version_id)
            .unwrap_or_else(|| {
                panic!(
                    "Version Name should always be available, but was not for {}",
                    x.version_id.0
                )
            })
            .clone();

        species_id_to_flavor_texts
            .entry(x.species_id)
            .or_default()
            .push(PokedexEntry::new(version_name, x.flavor_text));
    }
    species_id_to_flavor_texts
}

fn map_version_id_to_name(version_names: Vec<ApiVersionNames>) -> HashMap<VersionId, String> {
    let mut version_id_to_name: HashMap<VersionId, String> = HashMap::default();
    for x in version_names {
        if x.local_language_id != ENGLISH_LANGUAGE_ID {
            continue;
        }

        version_id_to_name.insert(x.version_id, x.name);
    }
    version_id_to_name
}

fn map_ability_id_to_names(ability_names: Vec<ApiAbilityName>) -> HashMap<AbilityId, String> {
    let mut ability_id_to_name: HashMap<AbilityId, String> = HashMap::default();
    for x in ability_names {
        if x.local_language_id != ENGLISH_LANGUAGE_ID {
            continue;
        }

        ability_id_to_name.insert(x.ability_id, x.name);
    }

    ability_id_to_name
}

pub fn load_csv<P: AsRef<Path>, T: DeserializeOwned>(path: P) -> Vec<T> {
    let mut results = Vec::new();

    let reader = csv::ReaderBuilder::new().has_headers(true).from_path(path);

    for record in reader.expect("").records().flatten() {
        let value: T = record.deserialize(None).expect("Unable to parse csv row");
        results.push(value);
    }

    results
}
