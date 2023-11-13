A discord bot to support playing a customized, Mystery-Dungeon Themed version of [Pokerole](https://www.pokeroleproject.com).

Since this is my first "bigger" rust project, and I'm still in the process of learning the language, things might not look very nice at times. This whole thing will be a never-ending, ongoing refactoring process. :D

## Environment Variables
We poll and combine data from different sources, this allows us to update the data inside the bot without requiring a new build for every minor addition. In order to get this to work, the bot requires the following environment variables to be set:
- **POKEMON_API** – path to a local clone of the [pokeAPI](https://github.com/PokeAPI/pokeapi) git repository
- **POKEROLE_DATA** – path to a local clone of the [Pokerole-Data](https://github.com/Pokerole-Software-Development/Pokerole-Data) git repository
- **CSV_DATA** – path to a local clone of the [Pokerole-Discord.py-Base](https://github.com/XShadeSlayerXx/PokeRole-Discord.py-Base) git repository
- **CUSTOM_DATA** – path to your custom data overrides. In our case, that's https://github.com/Jacudibu/pokerole-custom-data.
- **DISCORD_TOKEN** – the Discord Token for your bot.
- **DATABASE_URL** – URL to the SQLite database file.

## TODO
A small list of ideas which might be useful to explore in the future.
- Auto Backup DB nightly to discord channel on dev server.
- User Leave notifications - might be useful to have, dunno.
- Weather: Random Weather changes.
- Move usage tracking: What's a user's most used move?
- Track how much money the guild currently has
