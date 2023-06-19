use crate::game_data::parser::custom_data::custom_status_effect::CustomStatusEffect;
use crate::game_data::pokerole_discord_py_data::pokerole_discord_py_csv_parser::RawPokeStatus;

pub struct StatusEffect {
    pub name: String,
    pub description: String,
    pub resist: String,
    pub effect: String,
    pub duration: String,
}

impl StatusEffect {
    pub(in crate::game_data) fn new(raw: RawPokeStatus) -> Self {
        StatusEffect {
            name: raw.name,
            description: raw.description,
            resist: raw.resist,
            effect: raw.effect,
            duration: raw.duration
        }
    }

    pub(in crate::game_data) fn from_custom_data(raw: &CustomStatusEffect) -> Self {
        StatusEffect {
            name: raw.name.clone(),
            description: raw.description.clone(),
            resist: raw.resist.clone(),
            effect: raw.effect.clone(),
            duration: raw.duration.clone()
        }
    }

    pub(crate) fn build_string(&self) -> String {
        std::format!("### {}\n*{}*\n- {}\n- {}\n- {}",
                     &self.name,
                     &self.description,
                     &self.resist,
                     &self.effect,
                     &self.duration)
    }
}
