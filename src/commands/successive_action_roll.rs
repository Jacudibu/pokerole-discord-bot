use crate::Error;
use crate::commands::send_ephemeral_reply;
use crate::shared::PoiseContext;
use crate::shared::dice_rolls::ParsedRollQuery;
use rand::prelude::IndexedRandom;
use std::convert::Into;
use std::fmt::{Display, Formatter};
use std::ops::Not;

#[derive(PartialEq, Eq, Copy, Clone, Debug, poise::ChoiceParameter)]
pub enum SuccessiveActionKind {
    /// We roll as long as we don't fail.
    Successive,
    /// We always roll twice.
    Double,
    /// We always roll thrice.
    Triple,
}

impl SuccessiveActionKind {
    pub fn default_accuracy_reduction_per_throw(&self) -> u8 {
        match self {
            SuccessiveActionKind::Successive => 2,
            SuccessiveActionKind::Double => 1,
            SuccessiveActionKind::Triple => 1,
        }
    }

    pub fn maximum_hits(&self) -> u8 {
        match self {
            SuccessiveActionKind::Successive => u8::MAX,
            SuccessiveActionKind::Double => 2,
            SuccessiveActionKind::Triple => 3,
        }
    }
}

impl Display for SuccessiveActionKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SuccessiveActionKind::Successive => f.write_str("Successive"),
            SuccessiveActionKind::Double => f.write_str("Double"),
            SuccessiveActionKind::Triple => f.write_str("Triple"),
        }
    }
}

/// Roll multiple dice to quickly get the results for successive actions.
#[poise::command(slash_command)]
pub async fn successive_action_roll(
    ctx: PoiseContext<'_>,
    #[description = "What kind of successive action move are we using?"]
    action_kind: SuccessiveActionKind,
    #[description = "How many accuracy dies do we have initially?"]
    #[min = 1_u8]
    #[max = 20_u8]
    accuracy_dies: u8,
    #[description = "How many damage dies do we have initially?"]
    #[min = 0_i16]
    #[max = 40_i16]
    damage_dies: i16,
    #[description = "Override for the initial accuracy reduction. Defaults to 2 for Successive, 1 for Double/Triple."]
    #[min = 0_u8]
    #[max = 10_u8]
    base_accuracy_reduction: Option<u8>,
    #[description = "Override for how many accuracy dies are removed per success."]
    #[min = 1_u8]
    #[max = 100_u8]
    accuracy_reduction_per_success: Option<u8>,
    #[description = "Override for how many damage dies are added per success. May be negative."]
    #[min = -2_i8]
    #[max = 2_i8]
    damage_change_per_roll: Option<i8>,
) -> Result<(), Error> {
    let defer = ctx.defer();
    let accuracy_reduction_per_success = accuracy_reduction_per_success
        .unwrap_or(action_kind.default_accuracy_reduction_per_throw());
    let mut required_accuracy =
        1 + base_accuracy_reduction.unwrap_or(action_kind.default_accuracy_reduction_per_throw());
    let damage_change_per_roll = damage_change_per_roll.unwrap_or(0) as i16;

    let mut message = format!(
        "{action_kind} Action roll.\nParameters: Accuracy dies: {accuracy_dies} | Required Accuracy: {required_accuracy}+{accuracy_reduction_per_success} per success | Damage dies: {damage_dies}"
    );
    if damage_change_per_roll > 0 {
        message.push_str(&format!("+{damage_change_per_roll} per roll"));
    }
    message.push('\n');

    if required_accuracy > accuracy_dies {
        message.push_str("### That'd be an instant-miss! Did you typo your accuracy dies?");
        let _ = send_ephemeral_reply(&ctx, message).await;
        return Ok(());
    }

    let mut failed_successive_roll = false;
    let mut hit_success_count = 0;

    let mut roll_counter = 1;
    let mut crit_log = Vec::new();
    message.push_str("### Accuracy rolls:\n");
    while required_accuracy < accuracy_dies
        && failed_successive_roll.not()
        && roll_counter <= action_kind.maximum_hits()
    {
        let query = ParsedRollQuery::new(accuracy_dies.into(), None, None, true);
        let roll_result = query.execute();

        if roll_result.success_count >= required_accuracy {
            hit_success_count += 1;
        } else if action_kind == SuccessiveActionKind::Successive {
            failed_successive_roll = true;
        }

        message.push_str(&format!(
            "**Roll #{roll_counter}**: {} ({required_accuracy} needed)\n",
            roll_result.message.replace("\n", " – ")
        ));

        crit_log.push(roll_result.is_critical_hit);
        required_accuracy += accuracy_reduction_per_success;
        roll_counter += 1;
    }

    if hit_success_count == 0 {
        message.push_str(&format!(
            "**Missing completely!**\n*{}*",
            get_random_mockery(&COMPLETE_MISS_COMMENTARY)
        ));

        let _ = defer.await;
        let _ = ctx.reply(message).await;

        return Ok(());
    }
    if hit_success_count == 1 {
        message.push_str(&format!(
            "Succeeding once!\n*{}*\n",
            get_random_mockery(&SINGLE_HIT_COMMENTARY)
        ));
    } else {
        message.push_str(&format!(
            "Succeeding a total of **{hit_success_count}** times!\n"
        ));
    }

    let mut damage_dies = damage_dies;
    if damage_dies > 0 {
        let mut damage_success_count = 0;
        message.push_str("### Damage rolls:\n");
        for i in 0..hit_success_count {
            let damage_dies_for_this_roll = if damage_dies > u8::MAX as i16 {
                u8::MAX
            } else {
                damage_dies as u8
            };

            let query = ParsedRollQuery::new(damage_dies_for_this_roll.into(), None, None, false);
            let roll_result = query.execute();
            damage_success_count += roll_result.success_count as u16;

            let maybe_crit = if crit_log[i] { "(+CRIT)" } else { "" };

            message.push_str(&format!(
                "**Roll #{}**: {} {maybe_crit}\n",
                i + 1,
                roll_result.message.replace("\n", " – ")
            ));

            damage_dies += damage_change_per_roll;
        }

        message.push('\n');
        if hit_success_count == 0 {
            let hit_text = match hit_success_count {
                1 => "once",
                2 => "twice",
                3 => "thrice",
                x => &format!("{x} times"),
            };

            message.push_str(&format!(
                "**Hitting the target {hit_text}!**\n*{}*",
                get_random_mockery(&ZERO_DAMAGE_COMMENTARY)
            ));
        } else {
            let attack_string = if hit_success_count == 1 {
                "attack"
            } else {
                "attacks"
            };

            message.push_str(&format!("Yielding a total of **{damage_success_count}** successful damage rolls over {hit_success_count} {attack_string}!\n"));
            // message.push_str(&format!(
            //     "Final damage formula: `{damage_success_count} - ({hit_success_count} * Target Defense)` ... `+ {hit_success_count}xSTAB, item and type efficiency (if applicable)`"
            // ));
        }
    }

    let _ = defer.await;
    let _ = ctx.reply(message).await;
    Ok(())
}

fn get_random_mockery(from: &'static [&'static str]) -> &'static str {
    let mut rng = rand::rng();
    from.choose(&mut rng).expect("This should never be empty!")
}

const COMPLETE_MISS_COMMENTARY: [&str; 50] = [
    "Hey, that looked kinda funny!",
    "Huh, did they try to do something?",
    "Uh-oh!",
    "Gotta work a little harder on that hand-eye coordination!",
    "All bark, no bite!",
    "The wind felt that more than the target did.",
    "Swing and a miss!",
    "Their opponent yawns... dramatically.",
    "Whiff! Not even close!",
    "Somewhere, a coach is crying.",
    "They really committed to... whatever that was.",
    "They might have hit themselves by accident!",
    "Well, the floor took a lot of damage!",
    "That's one way to air out the arena.",
    "A noble effort. Very noble. Very missy.",
    "I think the target blinked and dodged all of it.",
    "They just invented a brand-new dance move!",
    "Spectators are unsure if that was an attack or performance art.",
    "A flawless display of enthusiasm!",
    "Clearly, this Pokémon is practicing social distancing.",
    "If they were aiming for the sky, they nailed it.",
    "It’s super effective—against empty space!",
    "Some attacks are dodged. This one... dodged itself.",
    "Their opponent didn’t even flinch.",
    "It's the thought that counts.",
    "If flailing was a strategy, they’d be winning!",
    "They missed everything but the spotlight.",
    "Air punches? Stylish. Effective? Not so much.",
    "That was some top-tier shadow boxing.",
    "Their attacks are evolving... into interpretive dance.",
    "A truly harmless rampage.",
    "I’ve seen tumbleweeds hit harder.",
    "They must've trained with a blindfold.",
    "Their energy was unmatched. Their accuracy? Not so much.",
    "You could almost hear a sad trombone playing.",
    "Did they just faint from embarrassment?",
    "Was that... a decoy attack? No? Oh.",
    "They’re just helping to stir the breeze.",
    "Even the announcer missed what that was supposed to be.",
    "That was more flail than fight.",
    "A full combo... of flops.",
    "I think the target waved back out of pity.",
    "New record for least threatening attack!",
    "That was less ‘battle’ and more ‘jazz hands’.",
    "It looked dangerous... for a second.",
    "On the bright side, nobody got hurt!",
    "Well, at least the ground is scared now.",
    "They missed with confidence. That’s something.",
    "Maybe they’re just building suspense?",
    "Now that’s what I call an aggressive warning shot!",
];

const ZERO_DAMAGE_COMMENTARY: [&str; 51] = [
    "All whilst looking very cute whilst doing so!",
    "Tickling the target vigorously in the process!",
    "Dealing mostly emotional damage... to themselves.",
    "It was a valiant effort, if nothing else.",
    "They made contact... with zero consequences!",
    "You could almost hear a ‘boop!’",
    "Impressively ineffective.",
    "Well, they tried. That’s what matters, right?",
    "A flurry of effort with absolutely no bite!",
    "Like punching a pillow with your eyes closed.",
    "They're gonna feel that... in their pride.",
    "Damage? Not even their hair got ruffled.",
    "More of a light breeze than an attack.",
    "That barely counted as a pat on the back.",
    "They might have annoyed their opponent. Maybe.",
    "At least they looked cool doing it. Sort of.",
    "That was... impressively harmless.",
    "An intense display of zero effectiveness!",
    "Their opponent is now slightly confused. Not hurt. Just confused.",
    "A demonstration of how not to hurt someone.",
    "They technically hit. Technically.",
    "It made contact—and that’s where the excitement ends.",
    "They left a mark! (An emotional one.)",
    "The spirit was willing, but the stats were weak.",
    "Hits delivered with all the force of a wet sponge.",
    "Even a feather would’ve done more.",
    "Their attacks have all the ferocity of a hug.",
    "At least they didn’t miss this time?",
    "You could almost hear the opponent laugh.",
    "It connected. It just... didn't matter.",
    "I think that attack healed their opponent’s confidence.",
    "Sometimes it’s the gesture that counts.",
    "They’ve won the award for Most Gentle Barrage.",
    "An attack so soft, it could be sold as a pillow.",
    "They’re really good at not hurting anything!",
    "Fierce energy, completely wasted.",
    "Even the dust didn't move.",
    "That was basically a massage.",
    "Harmless and oddly soothing.",
    "The target feels refreshed, actually.",
    "It did literally zero and metaphorical less.",
    "Not a scratch. Not a scare. Not a clue.",
    "They hit like a butterfly... that’s taking a nap.",
    "An audible ‘plink’ was heard. That’s it.",
    "Tried their best. Opponent didn’t notice.",
    "Even the move’s name was stronger than the result.",
    "Impact level: pillow fight.",
    "The Pokémon is now eligible to be a pacifist monk.",
    "On the upside, it was a perfect warm-up swing.",
    "If disappointment was damage, they'd be KO'd.",
    "Somewhere, a stat sheet is crying.",
];

const SINGLE_HIT_COMMENTARY: [&str; 51] = [
    "Well... one is better than none, right?",
    "Guess they just wanted to test the waters.",
    "Started strong, ended immediately.",
    "Came in like a wrecking ball... and left like a whisper.",
    "One lonely slap in a sea of missed chances.",
    "The most polite multi-attack I’ve ever seen.",
    "They brought a combo and used exactly 2% of it.",
    "One hit wonder, baby!",
    "They meant to do that. Probably.",
    "It’s called pacing. Very... slow pacing.",
    "All that build-up for a single tap.",
    "Their opponent is mildly inconvenienced.",
    "Like showing up to a buffet and eating one crouton.",
    "Talk about underachieving!",
    "They forgot the rest of the move existed.",
    "Just a teaser for what could’ve been.",
    "Their follow-through fell through.",
    "A thrilling solo performance in a supposed ensemble.",
    "One-hit combo! Technically still a combo!",
    "It’s not quantity, it’s... never mind.",
    "They did just enough to not be impressive.",
    "Blink and you missed it—no really, that was it.",
    "You call that a multi-hit?",
    "Putting the 'mini' in mini-barrage.",
    "It was supposed to be a flurry... not a flick.",
    "If hesitation were a strategy, they'd be champion.",
    "Hit once. Called it a day.",
    "Just a little sample of disappointment.",
    "They committed to the bare minimum.",
    "They started a combo and rage-quit halfway.",
    "That attack had performance issues.",
    "Even their Pokémon looked confused afterward.",
    "They brought a six-shooter and fired once.",
    "Like bringing fireworks and lighting one sparkler.",
    "That's one way to conserve energy!",
    "A minimalist approach to combat.",
    "Might as well have waved.",
    "They showed promise. Then revoked it.",
    "Their multi-attack needed a nap after the first hit.",
    "Dramatic pause... that never ended.",
    "One punch! And... existential crisis.",
    "If this were golf, they'd be winning.",
    "A bold one-hit strategy for a multi-hit move.",
    "One hit, infinite regret.",
    "They skipped the rest for suspense.",
    "Tried to combo, remembered they left the oven on.",
    "That was a preview, not a performance.",
    "The rest of the hits are on vacation.",
    "Tactical underdelivery.",
    "The first hit was lonely.",
    "A whisper of a combo. A hint of effort.",
];
