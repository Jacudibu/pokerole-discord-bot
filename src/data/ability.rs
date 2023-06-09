use serde::Deserialize;
use crate::data::pokerole_data::raw_ability::RawPokeroleAbility;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Ability {
    pub name: String,
    pub description: String,
    pub effect: String,
}

impl Ability {
    pub(in crate::data) fn new(raw: &RawPokeroleAbility) -> Self {
        Ability {
            name: raw.name.clone(),
            description: raw.description.clone(),
            effect: raw.effect.clone(),
        }
    }

    pub(crate) fn build_string(&self, suffix: &str) -> impl Into<String> + Sized {
        std::format!("### {} {}\n{}\n*{}*",
                     &self.name,
                     &suffix,
                     &self.effect,
                     &self.description)
    }
}
