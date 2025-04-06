use crate::shared::game_data::parser::custom_data::custom_ability::CustomAbility;
use crate::shared::game_data::pokerole_data::raw_ability::RawPokeroleAbility;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Ability {
    pub name: String,
    pub description: String,
    pub effect: String,
}

impl Ability {
    pub fn from_pokerole(raw: &RawPokeroleAbility) -> Self {
        Ability {
            name: raw.name.clone(),
            description: raw.description.clone(),
            effect: raw.effect.clone(),
        }
    }

    pub fn from_custom_data(raw: CustomAbility) -> Self {
        Ability {
            name: raw.name,
            description: raw.description,
            effect: raw.effect,
        }
    }

    pub fn build_string(&self, suffix: &str) -> impl Into<String> + Sized {
        std::format!(
            "### {} {}\n{}\n*{}*",
            &self.name,
            &suffix,
            &self.effect,
            &self.description
        )
    }
}
