pub struct Rule {
    pub name: String,
    pub flavor: Option<String>,
    pub text: String,
    pub example: Option<String>,
}

impl Rule {
    pub(crate) fn build_string(&self) -> impl Into<String> + Sized {
        let mut builder = serenity::utils::MessageBuilder::default();
        builder.push(std::format!("**{}**\n", &self.name));
        if let Some(flavor) = &self.flavor {
            builder.push_italic_line(flavor);
            builder.push('\n');
        }

        builder.push(&self.text);

        if let Some(example) = &self.example {
            builder.push('\n');
            builder.quote_rest();
            builder.push(std::format!("**Example**: {}", example));
        }

        builder.build()
    }
}

impl Rule {
    pub fn get_hardcoded_rules() -> Vec<Rule> {
        vec![
            Rule {
                name: String::from("Limit Break"),
                flavor: Some(String::from("By investing an extraordinary amount of effort, some pokemon can surpass their natural limits!")),
                text: String::from("You may spend (2 + Amount of previous Limit Breaks) stat points in order to increase your stats past your species' stat cap. For balancing reasons, you can never have more than 10 points in any particular stat, even by using this mechanic."),
                example: Some(String::from("\
Let's say your max Dexterity is 3. If you want to increase it to 4, you'll need to use two stat points.
Next up, you want to increase your Vitality past its limit. Since you've already used one limit break in the past, this would now cost 3 stat points.
If you stat is already at 10, you cannot limit break it any further."))
            }, Rule {
                name: String::from("Evolution"),
                flavor: Some(String::from("Sometimes they just grow a little too fast!")),
                text: String::from("\
You can evolve at any time or even start out as a fully evolved pokemon, but as long as you haven't reached the level required for your evolution yet, you will have to play with the base stats of your unevolved form. Evolution thresholds are Level 3 for second stage, and Level 6 for third stage evolutions. In severe cases where even the second evo has terrible stats, such as for e.g. the Weedle/Kakuna/Beedrill line, you may apply for an exception to be made.
If you want to have your evolution happen under specific circumstances, feel free to hit up any Quest Giver to create an evo-quest for you!

Characters who wish to not evolve upon reaching the required rank might choose to do so as well, and still receive the stats and most of the moves the evolved pokemon would have (not the abilities, though)! Think of it as an upgraded 'plus' version. This also does not lock you out from evolving at a later point!"),
                example: Some(String::from("Let's say you want to play a Tyranitar. Pupitar is probably not the most fun to play, so you decide to start out fully evolved right from the get go. Until level 3 you will have to play with the base stats of a Larvitar. Once you reach level 3, you can upgrade your base stats to that of a Pupitar, and, finally, once you reach level 6, your base stats may reflect those of a full-powered Tyranitar!"))
            }, Rule {
                name: String::from("Multi-Target and Area moves"),
                flavor: Some(String::from("Watch out, it's the 'Oops, I Did It Everywhere' attack!")),
                text: String::from("\
- Moves targeting **All Foes**: Declare the order in which your character focuses on your enemies.
- Moves targeting **Area**: Use `/select_random` to determine the order in which the combatants are hit. These moves target *everyone* except the user.

In both cases, only the first target can receive a critical hit, and for every successive target hit, reduce the damage die count by 1. There is *no* target limit."),
                example: Some(String::from("You are using Razor Leaf against three foes! First, you declare the order in which they are to be hit: Ursaring, Absol, Sneasler. Then, you roll accuracy. It's a crit, yay! Roll your regular 6 damage dies against the Ursaring. Critical damage will be applied! Then, roll 5 damage dies against the Lucario. Finally, 4 dies against the Sneasler. Both of those take the regular damage without crit modifiers."))
            }, Rule {
                name: String::from("Critical Strike"),
                flavor: Some(String::from("Hit 'em right between the legs!")),
                text: String::from("A critical strike occurs when you roll at least three 6 during an accuracy check (You need only 2 for 'High Critical' moves). After the damage reduction from defense, the damage dealt will be increased by 50%, rounded up. If the move applies stat boosts or reductions, those will be increased by 1."),
                example: Some(String::from("You crit and successfully roll 5 damage dies. Your enemy has 2 defense. The final damage dealt is (5 - 2) * 1.5 = 4.5, so 5 damage."))
            }, Rule {
                name: String::from("Levels and Ranks"),
                flavor: Some(String::from("Now, would you look at this shiny badge?")),
                text: String::from("\
A level up always requires 100 Experience Points. Experience is gained by going on quests and joining in on events, depending on duration and difficulty/danger levels! For each level up, you gain 1 Stat point, which you may freely spend to increase your stats (see Limit Breaking in case you are maxed out.)
You'll gain one combat stat point to allocate for each level up!

Furthermore, Levels decide your guild rank and when you can evolve:
Level 2 => Silver
Level 3 => Evo #1
Level 4 => Gold
Level 6 => Evo #2
Level 8 => Platinum
Level 16 => Diamond

Besides the extra die for all the `+ Rank` accuracy rolls, you'll gain 2 social stat points for each rank up!"),
                example: None
            }, Rule {
                name: String::from("Wound damage"),
                flavor: Some(String::from("It's just a flesh wound!")),
                text: String::from("Wound damage reduces your maximum HP and needs to be healed over time in base and/or by visiting a doctor. Your only way to heal wounds during a quest is by using Potion Items! If your max HP falls to 0, you lose consciousness and really need some medical assistance!"),
                example: None
            }, Rule {
                name: String::from("Stat Changes"),
                flavor: Some(String::from("Feeling weak... or... strong? I don't know how to feel.")),
                text: String::from("\
Stat changes done by different moves stack up to a maximum of +/- 3.
Critical stat changes apply/reduce one additional stat point, but not past the limit! If it would boost multiple stats, you got to choose one which will receive the crit bonus.
Defense cannot go below 0, combat stats can't go below 1."),
                example: None
            }, Rule {
                name: String::from("Alternative Moves (Dodge, Struggle, Help Another)"),
                flavor: Some(String::from("Gotta know the basics if you wanna be an adventurer!")),
                text: String::from("\
Every Character has access to the following moves at any time:
- Struggle
- Help Another
- Dodge\n"),
                example: None
            }, Rule {
                name: String::from("Accuracy Reduction"),
                flavor: Some(String::from("Got a little bit of dirt in your eye?")),
                text: String::from("Characters afflicted by accuracy reduction (either as a stat change or through a move effect) still roll the regular amount of accuracy dies, but then subtract -X from the amount of successes this roll had."),
                example: Some(String::from("Let's say you want to use Rock Slide. The move has a -1 Accuracy effect. You roll your regular Dex+Rank amount of dies for your accuracy and land two successes. Now we reduce those successes by -1... which leaves us with one success, so the attack still hits, yay!"))
            }, Rule {
                name: String::from("SFW / NSFW Channels"),
                flavor: Some(String::from("Keep it in your pants/sheath!")),
                text: String::from("Channels which are prefixed SFW are strictly SFW. Everywhere else, NSFW stuff is allowed (and optional), as long as it doesn't just randomly interrupt an ongoing scene without asking first."),
                example: None
            }, Rule {
                name: String::from("Rerolls"),
                flavor: Some(String::from("Lady Luck isn't smiling on you today, huh?")),
                text: String::from("Once per round, a player may expend one point of their Will to reroll an accuracy or chance dice roll. The reroll must be taken (Certain held items may change this) over the original roll."),
                example: None
            }, Rule {
                name: String::from("Defense"),
                flavor: Some(String::from("How much of a beating can you really take?")),
                text: String::from("You Physical Defense is `Vitality / 2`.\nYour Special Defense is `Insight / 2`.\nMinimum damage is always 1, unless you resist the attack's type - this is the only case where it gets reduced to 0."),
                example: Some(String::from("With 4 points in Vitality and 5 points in Insight, you'd get 2 physical and 3 special defense!"))
            }, Rule {
                name: String::from("Orre Character Import"),
                flavor: Some(String::from("Celebi did another woopsiedoodle!")),
                text: String::from("\
While everything that happened over in the Orre Region is considered non-canon since this is taking place in a different timeline and in the far future, you can import your characters from Orre over to this system!
If you where at least Silver Rank, your character may start with Level 2 here.
If you where Gold or higher, your character may start with Level 3.
All of these are optional, of course! And no items or other unlocks can be carried over.
It is fine to start out as an evolved mon here, just check `/rule name:Evolution` for the details.
Just follow the character creation guide and apply your two level ups afterwards, those will yield you one Combat Stat Point each and promote you to silver, which also yields two extra Social Stat points!"),
                example: None
            }, Rule {
                name: String::from("Character Slots"),
                flavor: Some(String::from("Sometimes you just need a friend! Sometimes that friend has to be yourself!")),
                text: String::from("\
You get one new character slot ever 5 levels (cumulative across all your characters!).
Using those or creating multiple characters is completely optional, of course.
Also, if you want more or need one earlier, you may request that from our admin team! We'll vote internally on a case-by-case basis to determine if it would work out."),
                example: None
            }, Rule {
                name: String::from("Successive Actions"),
                flavor: Some(String::from("And another one! And another one! And...")),
                text: String::from("\
Moves with Successive Actions hit multiple times. Every successive hit has a -2 accuracy reduction.
Keep rolling accuracy until the hit fails, then roll damage as often as you had successful accuracy rolls."),
                example: Some(String::from("\
Let's say you have a Dexterity of 5, are at Silver Rank and want to use Fury Swipes. That means you get 7 accuracy dice. We roll the following:
- Accuracy roll#1: 3 Successes! -> Hit (-1 base Accuracy from the move, but we still have 2 successes!)
- Accuracy roll#2: 4 Successes! -> Hit (Since this is the second action, we have an additional -2 accuracy here, but barely land the attack!)
- Accuracy roll#3: 3 Successes! -> Miss (The third roll has -4 Accuracy, so the attack doesn't connect.)

Now we roll damage twice, once for each hit. With a strength of 3, that means we'd just use `/roll` for 4d6 two times.
"))
            }, Rule {
                name: String::from("Combat"),
                flavor: None,
                text: String::from("Each Combat round is divided into two phases: Initiative and the actual fighting.
### Initiative
The round starts when the GM asks you to roll initiative! You do that by rolling one d6 and adding your Dexterity as flat amount on top.
> **Example**: if your Dexterity is 3, `/roll flat_addition: 3` will do the trick, as will `/r 1d6+3`

**If you have a move with `Priority` and want to use that, it's best to also mention this here. The highest priority move moves first!**

Also, if a player takes too long, they may be moved to the end of the turn order at the GMs discretion.
### Combat
After everyone rolled initiative, the turn order will be posted.

Once it's your turn, you can *optionally* interact with **one** item inside your backpack: this is the time to use a berry or switch your held item! Describe what your character will do and then use `/move` in order to post your action.

Next, roll the move's accuracy.
> **Example**: You are Bronze rank and have a dex of 3. `/move` says the move uses `Dexterity + Rank`, so you may roll 4 dice!

4-6 counts as success here. You need at least one success for the attack to hit. If you roll three 6's, you'll deal critical damage!

If the attack hits, do the same for damage.
> **Example**: You have a strength of 3. Your move damage calls for `Strength + 2`, so you may `/roll` 5 damage dies!

Every success will inflict one damage. STAB and super effectiveness both increase that damage by 1.

If the effects call for additional Chance Dice to cause status effects, roll those now in the order they are mentioned.
**Status effects only trigger when you roll a 6.**
"),
                example: None
            }
        ]
    }
}
