use crate::shared::game_data::ability::Ability;
use crate::shared::game_data::pokemon_api::pokemon_api_parser::ApiPokemonAbilities;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PokemonAbilities {
    pub ability1: String,
    pub ability2: Option<String>,
    pub hidden_ability: Option<String>,
    pub event_abilities: Option<String>,
}

impl From<&ApiPokemonAbilities> for PokemonAbilities {
    fn from(value: &ApiPokemonAbilities) -> Self {
        PokemonAbilities {
            ability1: value.ability1.clone(),
            ability2: value.ability2.clone(),
            hidden_ability: value.hidden.clone(),
            event_abilities: value.event.clone(),
        }
    }
}

impl PokemonAbilities {
    pub fn build_ability_string(
        &self,
        pokemon_emoji: &str,
        pokemon_name: &str,
        abilities: &HashMap<String, Ability>,
    ) -> impl Into<String> {
        let mut result = std::format!("## {}{} Abilities\n", pokemon_emoji, pokemon_name);
        Self::push_ability(&mut result, &self.ability1, abilities, "");
        if let Some(ability) = &self.ability2 {
            Self::push_ability(&mut result, ability, abilities, "");
        }

        if let Some(ability) = &self.hidden_ability {
            Self::push_ability(&mut result, ability, abilities, "(Hidden)");
        }

        if let Some(ability) = &self.event_abilities {
            Self::push_ability(&mut result, ability, abilities, "(Event / Hidden)");
        }

        result
    }

    pub fn build_simple_ability_list(&self, include_hidden: bool, include_event: bool) -> String {
        let mut result = format!("- {}\n", self.ability1);
        if let Some(ability) = &self.ability2 {
            result.push_str(&format!("- {}\n", ability));
        }

        if include_hidden {
            if let Some(ability) = &self.hidden_ability {
                result.push_str(&format!("- {} (Hidden)\n", ability));
            }
        }

        if include_event {
            if let Some(ability) = &self.event_abilities {
                result.push_str(&format!("- {} (Event)\n", ability));
            }
        }

        result
    }

    fn push_ability(
        result: &mut String,
        ability_name: &String,
        abilities: &HashMap<String, Ability>,
        suffix: &str,
    ) {
        match abilities.get(ability_name.to_lowercase().as_str()) {
            None => result.push_str(
                std::format!("### {} {}\nNot implemented. :(\n", ability_name, suffix).as_str(),
            ),
            Some(ability) => {
                result.push_str(std::format!("{}\n", ability.build_string(suffix).into()).as_str())
            }
        };
    }
}
