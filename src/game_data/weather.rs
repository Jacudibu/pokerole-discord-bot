use crate::game_data::parser::custom_data::custom_weather::CustomWeather;

#[derive(Debug, Clone)]
pub struct Weather {
    pub name: String,
    pub description: String,
    pub effect: String,
}

impl Weather {
    pub(crate) fn from_custom(raw: &CustomWeather) -> Weather {
        Weather {
            name: raw.name.clone(),
            description: raw.description.clone(),
            effect: raw.effect.clone(),
        }
    }

    pub(crate) fn build_string(&self) -> impl Into<String> + Sized {
        std::format!(
            "### {} Weather\n*{}*\n{}",
            &self.name,
            &self.description,
            &self.effect
        )
    }
}
