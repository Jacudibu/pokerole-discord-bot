use crate::shared::enums::PokemonType;
use crate::shared::game_data::pokemon::Pokemon;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;

pub struct TypeEfficiency {
    data: HashMap<PokemonType, HashMap<PokemonType, f32>>,
}

struct EfficiencyMapping {
    pokemon_type: PokemonType,
    efficiency: Efficiency,
}

impl Display for EfficiencyMapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pokemon_type)
    }
}

impl TypeEfficiency {
    pub fn new(data: HashMap<PokemonType, HashMap<PokemonType, f32>>) -> Self {
        TypeEfficiency { data }
    }

    pub fn get_type_resistances_string(&self, pokemon: &Pokemon, emoji: String) -> String {
        let efficiencies: Vec<EfficiencyMapping> = PokemonType::iter()
            .filter(|x| x != &PokemonType::Shadow)
            .map(|x| EfficiencyMapping {
                pokemon_type: x,
                efficiency: self.against_pokemon_as_enum(&x, pokemon),
            })
            .collect();

        let mut result = std::format!("## Type Efficiency against {}{}\n", emoji, pokemon.name);
        TypeEfficiency::print(&mut result, &efficiencies, Efficiency::SuperEffective);
        TypeEfficiency::print(&mut result, &efficiencies, Efficiency::Effective);
        // print(&mut result, &efficiencies, Efficiency::Normal);
        TypeEfficiency::print(&mut result, &efficiencies, Efficiency::Ineffective);
        TypeEfficiency::print(&mut result, &efficiencies, Efficiency::SuperIneffective);
        TypeEfficiency::print(&mut result, &efficiencies, Efficiency::Immune);

        result
    }

    fn print(result: &mut String, efficiencies: &[EfficiencyMapping], efficiency: Efficiency) {
        let filtered: Vec<String> = efficiencies
            .iter()
            .filter(|x| x.efficiency == efficiency && x.pokemon_type != PokemonType::Shadow)
            .map(|x| x.to_string())
            .collect();

        if filtered.is_empty() {
            return;
        }

        result.push_str(std::format!("### {}\n", efficiency).as_str());
        result.push_str(filtered.join("  |  ").as_str());
        result.push('\n');
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Efficiency {
    Normal,
    Ineffective,
    SuperIneffective,
    Effective,
    SuperEffective,
    Immune,
}

impl Display for Efficiency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Efficiency::Normal => "Normal",
            Efficiency::Ineffective => "Ineffective (-1)",
            Efficiency::SuperIneffective => "Super Ineffective (-2)",
            Efficiency::Effective => "Effective (+1)",
            Efficiency::SuperEffective => "Super Effective (+2)",
            Efficiency::Immune => "Immune (No Damage)",
        })
    }
}

impl TypeEfficiency {
    pub fn against_pokemon(&self, move_type: &PokemonType, pokemon: &Pokemon) -> f32 {
        let type1 = self
            .data
            .get(move_type)
            .unwrap()
            .get(&pokemon.types.type1)
            .unwrap();

        let type2 = match pokemon.types.type2 {
            None => &1.0,
            Some(t) => self.data.get(move_type).unwrap().get(&t).unwrap(),
        };

        type1 * type2
    }

    fn float_equals(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.1
    }

    pub fn against_pokemon_as_enum(
        &self,
        move_type: &PokemonType,
        pokemon: &Pokemon,
    ) -> Efficiency {
        let value = self.against_pokemon(move_type, pokemon);

        if TypeEfficiency::float_equals(value, 4.0) {
            return Efficiency::SuperEffective;
        }
        if TypeEfficiency::float_equals(value, 2.0) {
            return Efficiency::Effective;
        }
        if TypeEfficiency::float_equals(value, 1.0) {
            return Efficiency::Normal;
        }
        if TypeEfficiency::float_equals(value, 0.5) {
            return Efficiency::Ineffective;
        }
        if TypeEfficiency::float_equals(value, 0.25) {
            return Efficiency::SuperIneffective;
        }

        Efficiency::Immune
    }
}
