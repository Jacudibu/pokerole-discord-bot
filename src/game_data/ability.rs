use crate::game_data::parser::custom_data::custom_ability::CustomAbility;
use crate::game_data::pokerole_data::raw_ability::RawPokeroleAbility;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Ability {
    pub name: String,
    pub description: String,
    pub effect: String,
}

impl Ability {
    pub(in crate::game_data) fn from_pokerole(raw: &RawPokeroleAbility) -> Self {
        Ability {
            name: raw.name.clone(),
            description: raw.description.clone(),
            effect: raw.effect.clone(),
        }
    }

    pub(in crate::game_data) fn from_custom(raw: &CustomAbility) -> Self {
        Ability {
            name: raw.name.clone(),
            description: raw.description.clone(),
            effect: raw.effect.clone(),
        }
    }

    pub(crate) fn build_string(&self, suffix: &str) -> impl Into<String> + Sized {
        std::format!(
            "### {} {}\n{}\n*{}*",
            &self.name,
            &suffix,
            &self.effect,
            &self.description
        )
    }
}
