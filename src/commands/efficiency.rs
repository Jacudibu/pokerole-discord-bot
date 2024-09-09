use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{pokemon_from_autocomplete_string, Context, Error};
use crate::emoji;
use crate::enums::PokemonType;
use crate::game_data::pokemon::Pokemon;
use crate::game_data::type_efficiency::{Efficiency, TypeEfficiency};
use std::fmt;
use std::fmt::Formatter;
use strum::IntoEnumIterator;

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

struct EfficiencyMapping {
    pokemon_type: PokemonType,
    efficiency: Efficiency,
}

impl fmt::Display for EfficiencyMapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pokemon_type)
    }
}

pub fn get_type_resistances_string(
    pokemon: &Pokemon,
    emoji: String,
    type_efficiency: &TypeEfficiency,
) -> String {
    let efficiencies: Vec<EfficiencyMapping> = PokemonType::iter()
        .filter(|x| x != &PokemonType::Shadow)
        .map(|x| EfficiencyMapping {
            pokemon_type: x,
            efficiency: type_efficiency.against_pokemon_as_enum(&x, pokemon),
        })
        .collect();

    let mut result = std::format!("## Type Efficiency against {}{}\n", emoji, pokemon.name);
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
    let pokemon = pokemon_from_autocomplete_string(&ctx, &name)?;
    let emoji = emoji::get_any_pokemon_emoji_with_space(&ctx.data().database, pokemon).await;
    ctx.say(get_type_resistances_string(
        pokemon,
        emoji,
        &ctx.data().game.type_efficiency,
    ))
    .await?;

    Ok(())
}
