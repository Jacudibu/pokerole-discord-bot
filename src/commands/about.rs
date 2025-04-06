use crate::commands::Error;
use crate::shared::PoiseContext;

/// Blah blah blah
#[poise::command(slash_command)]
pub async fn about(ctx: PoiseContext<'_>) -> Result<(), Error> {
    // TODO: Might be funny to add an actual version string here once we use proper tags and git actions and such :>
    ctx.say("\
```k4rpOS v0x3b```
Made with love, cookies and <:ferris_happy:1120237205013663804>!
**You may use this bot on other discord servers as you see fit. Character data is server specific, so there's no need to worry about breaking anything!**
For help and support, check out the bot development & support discord server: <https://discord.gg/jVrv2YG2zU>

## Semi-Useful Links
- Source Code: [GitHub Repository](<https://github.com/Jacudibu/pokerole-discord-bot>)
- Custom Bot Data: [GitHub Repository](<https://github.com/Jacudibu/pokerole-custom-data>)
### External Data Sources
- **Original Pokerole System**: <https://www.pokeroleproject.com>
- Pokerole specific data is taken from **Pokerole-Data** [[Link](<https://github.com/Pokerole-Software-Development/Pokerole-Data>)]
- Raw pokemon data taken from **pokeAPI** [[Link](<https://github.com/PokeAPI/pokeapi>) – [License](<https://github.com/PokeAPI/pokeapi/blob/master/LICENSE.md>)]
Thanks to all of these projects for the effort they put in! <3
"
    ).await?;
    Ok(())
}
