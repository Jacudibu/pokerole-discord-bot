use crate::shared::game_data::parser::custom_data::custom_weather::CustomWeather;

#[derive(Debug, Clone)]
pub struct Weather {
    pub name: String,
    pub description: String,
    pub effect: String,
}

impl Weather {
    pub fn from_custom_data(raw: CustomWeather) -> Weather {
        Weather {
            name: raw.name,
            description: raw.description,
            effect: raw.effect,
        }
    }

    pub fn build_string(&self) -> impl Into<String> + Sized {
        std::format!(
            "### {} Weather\n*{}*\n{}",
            &self.name,
            &self.description,
            &self.effect
        )
    }
}
