use crate::game_data::pokerole_discord_py_data::pokerole_discord_py_csv_parser::RawPokeWeather;

#[derive(Debug)]
pub struct Weather {
    pub name: String,
    pub description: String,
    pub effect: String,
}

impl Weather {
    pub(in crate::game_data) fn new(raw: &RawPokeWeather) -> Self {
        Weather {
            name: raw.name.clone(),
            description: raw.description.clone(),
            effect: raw.effect.clone()
        }
    }

    pub(crate) fn build_string(&self) -> impl Into<String> + Sized {
        std::format!("### {}\n*{}*\n{}",
                     &self.name,
                     &self.description,
                     &self.effect)
    }
}
