use crate::commands::{Context, Error};

/// Blah blah blah
#[poise::command(slash_command)]
pub async fn about(
    ctx: Context<'_>,
) -> Result<(), Error> {
    // TODO: Might be funny to add an actual version string here once we use proper tags and git actions and such :>
    ctx.say("\
```k4rpOS v0x3b```
Made with love, cookies and <:ferris_happy:1120237205013663804>!
## Data Sources:
- **Original Pokerole System**: <https://www.pokeroleproject.com>
- Weather & Status Effects taken from **Pokerole-Discord.py-Base** [[Link](<https://github.com/XShadeSlayerXx/PokeRole-Discord.py-Base>)]
- All other Pokerole specific data is taken from **Pokerole-Data** [[Link](<https://github.com/Pokerole-Software-Development/Pokerole-Data>)]
- Raw pokemon data taken from **pokeAPI** [[Link](<https://github.com/PokeAPI/pokeapi>) – [License](<https://github.com/PokeAPI/pokeapi/blob/master/LICENSE.md>)]
Thanks to all of these projects for the effort they put in! <3
"
    ).await?;
    Ok(())
}
