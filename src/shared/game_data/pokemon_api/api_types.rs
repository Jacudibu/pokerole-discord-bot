use crate::shared::game_data::pokemon_api::PokemonApiId;
use serde::Deserialize;

#[derive(Debug, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub struct PokemonSpeciesId(pub u16);
#[derive(Debug, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub struct PokemonFormId(pub u16);
#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct AbilityId(pub u16);
#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct MoveId(pub u16);
#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct TypeId(pub u16);
#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct GenerationId(pub u8);
#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct LanguageId(pub u8);
#[derive(Debug, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VersionId(pub u8);

#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct MoveLearnMethodId(pub u8);

pub const LEVEL_UP: MoveLearnMethodId = MoveLearnMethodId(1);
pub const EGG: MoveLearnMethodId = MoveLearnMethodId(2);
pub const TUTOR: MoveLearnMethodId = MoveLearnMethodId(3);
pub const MACHINE: MoveLearnMethodId = MoveLearnMethodId(4);

/// version_groups.csv
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiVersionGroups {
    pub id: u8,
    pub identifier: String,
    pub generation_id: GenerationId,
    pub order: u8,
}

/// pokemon.csv
/// Contains base data about pokemon, such as height and weight. Pretty much just height and weight.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemon {
    pub id: PokemonApiId,
    pub identifier: String,
    pub species_id: PokemonSpeciesId,
    /// in 10cm
    pub height: u16,
    /// in 100g
    pub weight: u16,
    pub base_experience: Option<u16>,
    pub order: Option<u16>,
    pub is_default: u8,
}

/// ability_names.csv
/// Contains names for all abilities.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiAbilityName {
    pub ability_id: AbilityId,
    pub local_language_id: LanguageId,
    pub name: String,
}

/// pokemon_abilities.csv
/// Contains data about each pokemon's abilities.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemonAbility {
    pub pokemon_id: PokemonApiId,
    pub ability_id: AbilityId,
    pub is_hidden: u8,
    pub slot: u8,
}

/// pokemon_moves.csv
/// Contains info on what moves a pokemon can learn and how.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemonMoves {
    pub pokemon_id: PokemonApiId,
    pub version_group_id: u8,
    pub move_id: MoveId,
    pub pokemon_move_method_id: MoveLearnMethodId,
    pub level: u8,
    pub order: Option<u8>,
    //    pub mastery: Option<u8>,
}

/// pokemon_move_methods.csv
/// Maps a move is acquired
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemonMoveMethods {
    pub id: MoveLearnMethodId,
    pub identifier: String,
}

/// pokemon_species.csv
/// Contains general information regarding a pokemon species
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemonSpecies {
    pub id: PokemonSpeciesId,
    pub identifier: String,
    pub generation_id: GenerationId,
    pub evolves_from_species_id: Option<PokemonSpeciesId>,
    pub evolution_chain_id: u16,
    pub color_id: u16,
    pub shape_id: Option<u16>,
    pub habitat_id: Option<u16>,
    pub gender_rate: i16, // Genderless seems to be -1
    pub capture_rate: u16,
    pub base_happiness: Option<u8>,
    pub is_baby: u8,
    pub hatch_counter: Option<u8>,
    pub has_gender_differences: u8,
    pub growth_rate_id: u8,
    pub forms_switchable: u8,
    pub is_legendary: u8,
    pub is_mythical: u8,
    pub order: u16,
    pub conquest_order: Option<u16>,
}

/// pokemon_species_names.csv
/// Contains the name for regular Pokemon
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemonSpeciesNames {
    pub pokemon_species_id: PokemonSpeciesId,
    pub local_language_id: LanguageId,
    pub name: String,
    pub genus: String,
}

/// pokemon_forms.csv
/// Contains the names for regional Pokemon and other weird forms
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemonForm {
    pub id: PokemonFormId,
    pub identifier: String,
    pub form_identifier: Option<String>,
    pub pokemon_id: PokemonApiId,
    pub is_default: u8,
    pub is_battle_only: u8,
    pub is_mega: u8,
    pub form_order: u16,
    pub order: u16,
}

/// pokemon_form_names.csv
/// Contains the names for regional Pokemon and other weird forms
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemonFormNames {
    pub pokemon_form_id: PokemonFormId,
    pub local_language_id: LanguageId,
    pub form_name: String,
    pub pokemon_name: Option<String>,
}

/// pokemon_types.csv
/// Contains type identifiers for pokemon
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemonTypes {
    pub pokemon_id: PokemonApiId,
    pub type_id: TypeId,
    pub slot: u8,
}

/// pokemon_species_flavor_text.csv
/// Contains Pokedex entries for pokemon
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemonSpeciesFlavorText {
    pub species_id: PokemonSpeciesId,
    pub version_id: VersionId,
    pub language_id: LanguageId,
    pub flavor_text: String,
}

/// pokemon_form_types.csv
/// Contains type identifiers for pokemon
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiPokemonFormTypes {
    pub pokemon_form_id: PokemonFormId,
    pub type_id: TypeId,
    pub slot: u8,
}

/// move_names.csv
/// Contains the names for moves
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiMoveNames {
    pub move_id: MoveId,
    pub local_language_id: LanguageId,
    pub name: String,
}

/// type_efficacy.csv
/// Tells us how much damage an attack will deal against a certain single type
#[derive(Debug, Deserialize)]
pub struct ApiTypeEfficacy {
    pub damage_type_id: TypeId,
    pub target_type_id: TypeId,
    pub damage_factor: u8,
}

/// version_names.csv
/// Tells us how pokemon versions are called by humans
#[derive(Debug, Deserialize)]
pub struct ApiVersionNames {
    pub version_id: VersionId,
    pub local_language_id: LanguageId,
    pub name: String,
}
