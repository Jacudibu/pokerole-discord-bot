A discord bot to support playing a homebrewed, Mystery-Dungeon Themed version
of [Pokèrole](https://www.pokeroleproject.com).

If you are fine with our custom changes to the Pokèrole Systems, feel free to
just [add it to your own server using this link](https://discord.com/oauth2/authorize?client_id=1113153708201615430&permissions=9089493346368&scope=applications.commands%20bot)!
Any persistent data is server specific, so there's no need to worry about breaking anything.

# Features

Quickly get an overview of any Pokèmon's stats, moves, abilities...
![Stat Screenshot](screenshots/stats.png)
![Stat Screenshot](screenshots/moves.png)

Create and keep track of your characters with the bot!
![Stat Screenshot](screenshots/character_post.png)
![Stat Screenshot](screenshots/character_edit.png)

Lots of tiny quality of life things

- View all moves a pokemon can learn in the games through TMs/Tutors/Breeding
- Type effectiveness charts
- Automatic Emoji upload for newly created characters
- Automated database backup into a discord channel of your choice
- Separate Wallets for shops, guilds, NPCs...
  ...and a bunch more, pretty much anything to run your Pokerole campaign!

# Bot Commands

You can browse all commands and their descriptions in a discord server by just entering /, so keeping an updated list
with every command under the sun here is a little silly, but here's the most important ones for running a server.

**It might be best to browse the Command list in `Server Settings -> Integrations -> Manage` after adding the bot to
your server.** You can set custom overrides on which roles can access specific commands there. By default, anything that
changes stuff in the Database is only available to admins.

- `/setup_guild`: Used to set up some specific bits for your server.
- `/create_character`: Creates characters. This will also create emojis for them, if there isn't one already.
- `/edit_character`: Allows you to change pretty much anything about a character.
- `/reward_[experience/money/...]`: Add stuff to a character. Some of these are fairly specific to the main server this
  bot was initially created for, just ignore them if you don't need them.
- `/create_quest`: Creates Quests. We recommend using a single forum thread per quest to keep things organized. There's
  a built-in failsafe to ensure this won't be used in a used channel.

To quickly look up things, there are `/pokemon`, `/ability`, `/move` and `/item`.

# Contributing

Contributions of any kind are always welcome!
If it's a bigger feature just let me know what you want to work on by creating an issue first, to avoid accidental
duplicate work.
Also feel free to join the [development discord server](https://discord.gg/jVrv2YG2zU) by clicking this fancy button:

[![Discord](https://img.shields.io/discord/1115690620342763645.svg?logo=discord&logoColor=white&logoWidth=20&labelColor=7289DA&label=Discord&color=17cf48)](https://discord.gg/wf7eUEBk9w)

# Running the bot yourself

As long as you got a working rust environment set up, this should be fairly straightforward.

### Environment Variables

We poll and combine data from different sources, this allows us to update the data inside the bot without requiring a
new build for every minor addition. In order to get this to work, the executable requires the following environment
variables to be set:

- **POKEMON_API** – path to a local clone of the [pokeAPI](https://github.com/PokeAPI/pokeapi) git repository
- **POKEMON_API_SPRITES** - Optional. Path to a local clone of the [pokeAPI Sprites](https://github.com/PokeAPI/sprites)
  git
  repository. Only needed if you want to generate emojis for your characters.
- **POKEROLE_DATA** – path to a local clone of
  the [Pokerole-Data](https://github.com/Pokerole-Software-Development/Pokerole-Data) git repository
- **CUSTOM_DATA** – path to your custom data overrides. In our case,
  that's https://github.com/Jacudibu/pokerole-custom-data.
- **DISCORD_TOKEN** – the Discord Token for your bot.
- **DATABASE_URL** – URL to the SQLite database file.
- **DB_BACKUP_CHANNEL_ID** – Optional. Discord Channel ID into which daily backups should be posted.
- **STATUS_CHANNEL_ID** - Optional. Discord Channel ID for bot status updates.
