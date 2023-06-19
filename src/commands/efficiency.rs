use std::fmt;
use std::fmt::Formatter;
use strum::IntoEnumIterator;
use crate::commands::{Context, Error};
use crate::commands::autocompletion::autocomplete_pokemon;
use crate::game_data::pokemon::Pokemon;
use crate::game_data::type_efficiency::{Efficiency, TypeEfficiency};
use crate::enums::PokemonType;

fn print(result: &mut String, efficiencies: &[EfficiencyMapping], efficiency: Efficiency) {
    let filtered: Vec<String> = efficiencies.iter()
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

struct EfficiencyMapping {
    pokemon_type: PokemonType,
    efficiency: Efficiency,
}

impl fmt::Display for EfficiencyMapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pokemon_type)
    }
}

pub fn get_type_resistances_string(pokemon: &Pokemon, type_efficiency: &TypeEfficiency) -> String {
    let efficiencies: Vec<EfficiencyMapping> = PokemonType::iter()
        .filter(|x| x != &PokemonType::Shadow)
        .map(|x| EfficiencyMapping {pokemon_type: x, efficiency: type_efficiency.against_pokemon_as_enum(&x, pokemon)})
        .collect();

    let mut result = std::format!("## Type Efficiency against {}\n", pokemon.name);
    print(&mut result, &efficiencies, Efficiency::SuperEffective);
    print(&mut result, &efficiencies, Efficiency::Effective);
    // print(&mut result, &efficiencies, Efficiency::Normal);
    print(&mut result, &efficiencies, Efficiency::Ineffective);
    print(&mut result, &efficiencies, Efficiency::SuperIneffective);
    print(&mut result, &efficiencies, Efficiency::Immune);

    result
}

/// Display status effects
#[poise::command(slash_command)]
pub async fn efficiency(
    ctx: Context<'_>,
    #[description = "Get a typechart for a certain mon."]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
) -> Result<(), Error> {
    if let Some(pokemon) = ctx.data().game.pokemon.get(&name.to_lowercase()) {
        ctx.say(get_type_resistances_string(pokemon, &ctx.data().game.type_efficiency)).await?;
    } else {
        ctx.send(|b| {
            b.content(std::format!("Unable to find a pokemon named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name));
            b.ephemeral(true)
        }).await?;
    }

    Ok(())
}
