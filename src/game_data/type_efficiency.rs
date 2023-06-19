use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::game_data::pokemon::Pokemon;
use crate::enums::PokemonType;

pub struct TypeEfficiency {
    data: HashMap<PokemonType, HashMap<PokemonType, f32>>
}

impl TypeEfficiency {
    pub fn new(data: HashMap<PokemonType, HashMap<PokemonType, f32>>) -> Self {
        TypeEfficiency {data}
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
            Efficiency::Immune => "Immune (No Damage)"
        })
    }
}

impl TypeEfficiency {
    pub fn against_pokemon(&self, move_type: &PokemonType, pokemon: &Pokemon) -> f32 {
        let type1 = self.data.get(move_type).unwrap().get(&pokemon.type1).unwrap();

        let type2 = match pokemon.type2 {
            None => &1.0,
            Some(t) => self.data.get(move_type).unwrap().get(&t).unwrap()
        };

        type1 * type2
    }

    fn float_equals(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.1
    }

    pub fn against_pokemon_as_enum(&self, move_type: &PokemonType, pokemon: &Pokemon) -> Efficiency {
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
