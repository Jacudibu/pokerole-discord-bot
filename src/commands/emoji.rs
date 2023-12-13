use crate::commands::autocompletion::autocomplete_pokemon;
use crate::commands::{Context, Error};
use crate::enums::PokemonGeneration;
use crate::game_data::pokemon::Pokemon;
use poise::CreateReply;

const GEN5_ANIMATED: &str = "https://github.com/PokeAPI/sprites/blob/master/sprites/pokemon/versions/generation-v/black-white/animated/";
const GEN5_ANIMATED_FEMALE: &str = "https://github.com/PokeAPI/sprites/blob/master/sprites/pokemon/versions/generation-v/black-white/animated/female/";

const FRONT_MALE: &str = "https://github.com/PokeAPI/sprites/blob/master/sprites/pokemon/";
const FRONT_FEMALE: &str = "https://github.com/PokeAPI/sprites/blob/master/sprites/pokemon/female/";

fn build_string(pokemon: &Pokemon) -> String {
    let mut result = String::from("");

    if pokemon.species_data.generation <= PokemonGeneration::Five {
        result.push_str(
            std::format!(
                "\
## Animated Gen 5 sprite
Male/Unisex: <{}{}.gif>\n",
                GEN5_ANIMATED,
                pokemon.poke_api_id.0
            )
            .as_str(),
        );

        if pokemon.species_data.has_gender_differences {
            result.push_str(
                std::format!(
                    "Female: <{}{}.gif>\n",
                    GEN5_ANIMATED_FEMALE,
                    pokemon.poke_api_id.0
                )
                .as_str(),
            );
        }
    }

    result.push_str(
        std::format!(
            "\
## Regular Front Sprite
Male/Unisex: <{}{}.png>\n",
            FRONT_MALE,
            pokemon.poke_api_id.0
        )
        .as_str(),
    );

    if pokemon.species_data.has_gender_differences {
        result.push_str(
            std::format!("Female: <{}{}.png>\n", FRONT_FEMALE, pokemon.poke_api_id.0).as_str(),
        );
    }

    result.push_str("\n\n**When adding the emoji to the server, make sure to cut out the whitespace around the sprite, and make it square sized so discord doesn't stretch it in some awkward way.**");

    result
}

/// Display links to fancy emojis!
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn emoji(
    ctx: Context<'_>,
    #[description = "Which pokemon?"]
    #[rename = "pokemon"]
    #[autocomplete = "autocomplete_pokemon"]
    name: String,
) -> Result<(), Error> {
    if let Some(pokemon) = ctx.data().game.pokemon.get(&name.to_lowercase()) {
        ctx.say(build_string(pokemon)).await?;
    } else {
        ctx.send(CreateReply::default()
            .content(std::format!("Unable to find a pokemon named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", name))
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}
