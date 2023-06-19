use crate::game_data::parser::custom_data::custom_potion::CustomPotion;

#[derive(Debug)]
pub struct Potion {
    pub name: String,
    pub description: String,
    pub effect: String,
    pub recipes: String,
}

impl Potion {
    pub(crate) fn build_string(&self) -> impl Into<String> + Sized {
        std::format!("### {}\n*{}*\n{}\n**Recipes**:\n{}", &self.name, &self.description, &self.effect, &self.recipes)
    }
}

impl Potion {
    pub(in crate::game_data) fn from_custom_data(raw: &CustomPotion) -> Self {
        Potion {
            name: raw.name.clone(),
            description: raw.description.clone(),
            effect: raw.effect.clone(),
            recipes: Potion::parse_recipes(&raw.recipes),
        }
    }

    fn parse_recipes(raw: &Vec<String>) -> String {
        let mut result = String::new();
        for recipe in raw {
            result.push_str("- ");
            result.push_str(recipe.as_str());
            result.push('\n');
        }

        result
    }
}
