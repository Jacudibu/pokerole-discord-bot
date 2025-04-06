use crate::shared::game_data::pokerole_data::raw_nature::RawPokeroleNature;

#[derive(Debug, Clone)]
pub struct Nature {
    pub name: String,
    pub keywords: String,
    pub description: String,
}

impl Nature {
    pub fn build_string(&self) -> impl Into<String> + Sized {
        std::format!(
            "### {}\n**Keywords**: {}\n*{}*",
            &self.name,
            &self.keywords,
            &self.description
        )
    }
}

impl Nature {
    pub fn from_pokerole(raw: &RawPokeroleNature) -> Self {
        Nature {
            name: raw.name.clone(),
            keywords: raw.keywords.clone(),
            description: raw.description.clone(),
        }
    }
}
