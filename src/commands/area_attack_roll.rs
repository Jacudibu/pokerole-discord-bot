use crate::Error;
use crate::commands::{attack_roll, select_random, send_ephemeral_reply};
use crate::shared::dice_rolls::ParsedRollQuery;
use crate::shared::utility::message_splitting::split_long_messages;
use crate::shared::{PoiseContext, dice_rolls};
use std::convert::Into;

/// Roll multiple dice to quickly get the results for area attacks.
#[poise::command(slash_command)]
#[allow(clippy::too_many_arguments)]
pub async fn area_attack_roll(
    ctx: PoiseContext<'_>,
    #[description = "How many accuracy dies should be rolled?"]
    #[min = 1_u8]
    #[max = 40_u8]
    accuracy_dies: u8,
    #[description = "How many damage dies should be rolled?"]
    #[min = 0_i8]
    #[max = 80_i8]
    damage_dies: i8,
    #[description = "A comma,separated,list of names for your targets. We can do up to 20 at once!"]
    targets: String,
    #[description = "How many 6's are required to crit."]
    #[min = 0_u8]
    #[max = 5_u8]
    crit_6_count: Option<u8>,
    #[description = "How many status effect dies should be rolled?"]
    #[min = 0_u8]
    #[max = 5_u8]
    status_effect_dies: Option<u8>,
    #[description = "How many status effect dies should be rolled for a second status effect?"]
    #[min = 0_u8]
    #[max = 5_u8]
    status_effect_dies_2: Option<u8>,
    #[description = "Add an accuracy reduction. Defaults to 0."]
    #[min = 1_u8]
    #[max = 10_u8]
    accuracy_reduction: Option<u8>,
) -> Result<(), Error> {
    let defer = ctx.defer();
    let required_accuracy = 1 + accuracy_reduction.unwrap_or(0);
    let crit_6_count = crit_6_count.unwrap_or(dice_rolls::DEFAULT_CRIT_DIE_COUNT);

    let mut message = format!(
        "### Area Attack roll.\nParameters: Accuracy dies: {accuracy_dies} | Required Accuracy: {required_accuracy} | Damage dies: {damage_dies}"
    );
    dice_rolls::append_crit_stat_if_changed(&mut message, crit_6_count);

    message.push_str("\n\n");

    if required_accuracy > accuracy_dies {
        message.push_str("### That'd be an instant-miss! Did you typo your accuracy dies?");
        let _ = send_ephemeral_reply(&ctx, message).await;
        return Ok(());
    }

    let query = ParsedRollQuery::new(accuracy_dies.into(), None, None, Some(crit_6_count));
    let accuracy_roll_result = query.execute();

    message.push_str(&format!(
        "**Accuracy roll**: {} ({required_accuracy} needed)\n",
        accuracy_roll_result.message.replace("\n", " – ")
    ));

    if required_accuracy > accuracy_roll_result.success_count {
        attack_roll::append_random_mockery(&mut message, &COMPLETE_MISS_COMMENTARY);
        let _ = defer.await;
        let _ = ctx.reply(message).await;

        return Ok(());
    }

    attack_roll::append_random_mockery(&mut message, &RANDOM_TARGET_ORDER_COMMENTARY);
    message.push('\n');

    let targets = select_random::get_randomized_elements_from_csv(None, targets);
    let mut is_first_hit = true;
    let mut damage_dies = damage_dies;
    for target in targets {
        message.push_str(&format!("**Targeting {target}!**\n"));

        if damage_dies > 0 {
            let query = ParsedRollQuery::new(Some(damage_dies as u8), None, None, None);
            let damage_roll_result = query.execute();

            let maybe_crit = if is_first_hit && accuracy_roll_result.is_critical_hit {
                "(+CRIT)"
            } else {
                ""
            };

            message.push_str(&format!(
                "> **Damage roll**: {} {maybe_crit}\n",
                damage_roll_result.message.replace("\n", " – ")
            ));
        }

        attack_roll::append_status_effect_roll(status_effect_dies, "> ", "", &mut message);
        attack_roll::append_status_effect_roll(status_effect_dies_2, "> ", "#2 ", &mut message);
        is_first_hit = false;
        damage_dies -= 1;
    }

    let _ = defer.await;

    for message in split_long_messages(message) {
        let _ = ctx.reply(message).await;
    }
    Ok(())
}

const COMPLETE_MISS_COMMENTARY: [&str; 52] = [
    "Congratulations! You've just created a minor natural disaster — for fun!",
    "All that for nothing.",
    "That looked expensive... and pointless.",
    "A dramatic display of failure.",
    "Big move. Zero impact.",
    "They went all-in and got absolutely nothing.",
    "So much noise, so little result.",
    "It hit no one, but it sure looked cool!",
    "They just emptied the clip into the void.",
    "The only thing affected was the vibe.",
    "An impressive effort to scare the air molecules.",
    "That was basically a special effects test.",
    "They announced their presence. And nothing else.",
    "The opponents remain entirely unbothered.",
    "A bold move to miss everyone.",
    "An overcommitment to the art of missing.",
    "The arena's ears are ringing. That’s about it.",
    "They must’ve been aiming for future enemies.",
    "Who needs accuracy when you have spectacle?",
    "That was a cinematic failure.",
    "I think they just wanted attention.",
    "A big swing at absolutely no one.",
    "They activated a light show — not an attack.",
    "Did they mean to hit anyone, or...?",
    "A very convincing threat. Not much else.",
    "Spectacular. Useless. Memorable.",
    "The enemies dodged. Or maybe just stood still.",
    "Their intimidation stat is high. Their accuracy? Not so much.",
    "A full-power group miss!",
    "You could feel the effort. Just not the results.",
    "They really committed to disappointing everyone equally.",
    "A great way to clear the air... and nothing else.",
    "Not a single soul was touched.",
    "They missed so hard it looped back to impressive.",
    "That was like shouting in a crowd — dramatic, but ineffective.",
    "An inspired attempt to hit everything... except the targets.",
    "It echoed across the field... but that's all.",
    "Somewhere, a coach is rethinking their life choices.",
    "A masterclass in wasted potential.",
    "If the goal was to create suspense, mission accomplished.",
    "A full-area threat with none of the consequences.",
    "That move was heard around the world — and dodged by all of it.",
    "The power was real. The accuracy was theoretical.",
    "It felt important. It wasn’t.",
    "They just gave a TED Talk in the middle of a battle.",
    "A wide miss for wide audiences.",
    "Their opponents didn't even flinch.",
    "All that buildup... for that?",
    "They attacked the concept of enemies, not the actual ones.",
    "It’s hard to miss that much. Truly.",
    "The effort was massive. The failure was larger.",
    "Impressive in size. Legendary in futility.",
];

const RANDOM_TARGET_ORDER_COMMENTARY: [&str; 153] = [
    "Let’s find out who gets hit first — completely fair, totally random!",
    "Spinning the wheel of unfortunate priorities...",
    "Which poor soul goes first? Only fate knows!",
    "Time to shuffle the order of doom!",
    "No favorites here — the attack order is up to chance!",
    "Deciding who eats the first hit... randomly!",
    "Let’s randomize the pain delivery route!",
    "Who gets clobbered first? Let’s ask the dice.",
    "Distributing damage in random order — equal opportunity chaos!",
    "Rolling to see who gets unlucky first...",
    "It’s not personal — just randomized!",
    "We’ll let fate choose the lucky first victim!",
    "The attack queue is getting shuffled!",
    "It’s like a lottery... but with more bruising.",
    "Let’s spin the pain wheel and see where it lands first!",
    "No bias here — just good old chaos ordering.",
    "Pulling names from the hat of suffering...",
    "Deciding the order of regret!",
    "Shuffling the hit list...",
    "Time to deal damage... randomly and democratically!",
    "The RNG will now choose who gets whacked first!",
    "Prioritizing targets? Nah, we let fate do that.",
    "Rearranging targets like a deck of doom!",
    "Starting a fair and balanced beatdown... in random order!",
    "Which target gets the honor of being first? Let’s find out!",
    "Rolling for hit order — place your bets!",
    "Target order: determined by pure chaos!",
    "Nobody’s safe — but someone’s going first!",
    "Launching attack... with shuffled priorities!",
    "Time to deal damage, randomized for flavor!",
    "Target priority? Never heard of it!",
    "Lining up the targets... badly and randomly.",
    "It’s time for the damage lottery!",
    "Someone’s gotta go first. Let’s make it random!",
    "Eeny, meeny, miny — chaos.",
    "Randomizing the victim queue!",
    "Serving up damage — order chosen by dice gods.",
    "The hit parade begins… in random order!",
    "It’s a mystery who goes down first!",
    "Hope someone packed insurance — order’s out of our hands!",
    "Let’s pick the unlucky lead-off target!",
    "Damage roulette begins now!",
    "Which target gets the spotlight first? Let’s ask chance.",
    "The attack’s coming... just not sure who gets it first!",
    "We let the dice pick who gets regret first!",
    "Organizing pain... with maximum disorder.",
    "Time to roll for first target!",
    "It’s chaos o’clock — let’s decide the strike order!",
    "There is no plan. Only target shuffling.",
    "Let the drama of random targeting begin!",
    "No tactics. Just vibes.",
    "Reordering targets by the power of shrug!",
    "Rolling initiative, but for suffering!",
    "Someone has to go first. RNG will decide their fate.",
    "Time to put these targets in random firing order!",
    "Step right up — who wants to be unlucky today?",
    "Damage is coming — we just don’t know where it starts.",
    "No strategy here. Just pure chaos sequencing!",
    "The hit list is now being randomized...",
    "Choosing a victim sequence... the chaotic way!",
    "Don’t take it personally. The dice just hate you.",
    "We’re going to let fate figure out the first casualty.",
    "Fate is loading the damage queue...",
    "It’s time to let luck be the tactician!",
    "Spinning the order wheel of misfortune!",
    "Let’s roll for regret sequencing!",
    "Hope you weren’t expecting fairness in order!",
    "Determining strike order with scientific randomness!",
    "Let’s get the randomness rolling — literally!",
    "Who gets hit first? Not even I know!",
    "Hold tight — we’re shuffling the target deck!",
    "This will be completely impartial chaos. Promise!",
    "Prioritizing targets? That’s so last turn.",
    "The damage train is leaving — who’s the first stop?",
    "No one’s safe, but someone’s first. Let’s find out!",
    "We asked a coin, a die, and a squirrel. Consensus reached!",
    "The strike order is now chaos-certified!",
    "Rolling dice to see who regrets their life choices first...",
    "The RNG wheel spins! Screaming optional!",
    "And now... a completely irresponsible order of destruction!",
    "We’ll be attacking in absolutely no logical order!",
    "No strategy here — just vibes and mild chaos.",
    "And the first sacrifice shall be... decided randomly!",
    "Time to find out who annoyed the universe the most.",
    "Step right up for your randomized whacking!",
    "Someone’s about to get hit first — and it might be personal!",
    "It’s like a raffle, but the prize is pain!",
    "We asked the dice who should suffer first. The dice were cruel.",
    "Starting the attack sequence... in comedy mode!",
    "Hope someone brought a helmet — this is pure improv!",
    "Attack order? Never heard of her.",
    "Let’s put targets in random order — for dramatic tension!",
    "Picking a victim at random, like a badly managed game show!",
    "Who gets hit first? Spin the Wheel of Misfortune™!",
    "Just a moment — the chaos monkeys are deciding...",
    "Prioritizing by who looked at us funny.",
    "Tossing names into a blender and seeing who gets puree'd first!",
    "Initiating random violence... now!",
    "Throwing darts at the target list again!",
    "Hold on, consulting a crystal ball and some dice.",
    "Who goes first? It’s whoever the dice hate today!",
    "Performing randomized target acquisition. Science!",
    "Shuffling targets like a very angry DJ.",
    "You get a hit! You get a hit! But who gets it first?",
    "Initiating completely fair and mildly cursed hit order!",
    "Organizing pain distribution alphabetically! Just kidding — RANDOM!",
    "And now… a surprise attack order, courtesy of our gremlin intern!",
    "Unleashing chaos. Gently. One target at a time.",
    "Let’s see who RNG woke up cranky about!",
    "Starting the attack with a random act of hostility.",
    "Time to disappoint someone specifically... by chance!",
    "Deciding attack order by pulling names out of a suspicious hat.",
    "Sorting targets based on who tripped last turn.",
    "Spin the wheel! Win a concussion!",
    "Starting the pain parade with a random guest of honor!",
    "It’s time to play: Who Gets Hit First?",
    "The first lucky victim is being selected now!",
    "RNG, shuffle these souls like a bad playlist!",
    "And the chaos gods said... this one dies first!",
    "Commencing randomized smacking sequence!",
    "Assigning first place in the hurt line!",
    "The order of destruction is... drumroll of doom...",
    "Our target queue is being generated by trained raccoons.",
    "Drawing straws for first blood!",
    "Distributing damage via a lottery no one wanted to enter.",
    "Let’s pick someone to regret existing — completely at random!",
    "Target order determined by throwing darts blindfolded.",
    "Hold onto your hit points, we’re choosing randomly!",
    "Selecting the first target with a blindfold and a grudge!",
    "Let fate decide who goes down slightly faster!",
    "Who’s up first in the suffering queue? Let’s find out!",
    "Somebody’s about to be first on the punch list!",
    "Our sorting hat is just a dice in a trench coat.",
    "Brace yourself — chaos is making decisions now.",
    "Attack order courtesy of 'Spinny the RNG Goblin'!",
    "Choosing targets with maximum confusion — on purpose!",
    "Who gets hit first? We let the hamster in charge of RNG decide!",
    "It’s random, it’s chaotic, it’s deeply concerning!",
    "Let’s roll some dice and ruin someone’s day — efficiently!",
    "Selecting the unlucky first target… doom pending.",
    "First target? Whichever one flinched first!",
    "Attack sequence determined by imaginary astrology!",
    "Your order of suffering is now being randomized.",
    "Placing bets on who gets walloped first!",
    "We asked RNG to be gentle. RNG declined.",
    "Opening the buffet of pain — who’s first in line?",
    "Queueing up targets in a totally nonsensical order!",
    "Today’s pain is brought to you by random.org!",
    "And the RNG says: you!",
    "We let fate handle the hard choices. Bad idea, really.",
    "Initiating the tactical dartboard method!",
    "Let’s deal damage the lazy, chaotic way!",
    "It's attack time — let’s go full random feral mode!",
];
