use crate::Error;
use crate::commands::send_ephemeral_reply;
use crate::shared::dice_rolls::ParsedRollQuery;
use crate::shared::{PoiseContext, dice_rolls};
use rand::prelude::IndexedRandom;
use std::convert::Into;

/// Roll multiple dice to quickly get the results for successive actions.
#[poise::command(slash_command)]
pub async fn attack_roll(
    ctx: PoiseContext<'_>,
    #[description = "How many accuracy dies should be rolled?"]
    #[min = 1_u8]
    #[max = 20_u8]
    accuracy_dies: u8,
    #[description = "How many damage dies should be rolled?"]
    #[min = 0_u8]
    #[max = 40_u8]
    damage_dies: u8,
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
        "### Attack roll.\nParameters: Accuracy dies: {accuracy_dies} | Required Accuracy: {required_accuracy} | Damage dies: {damage_dies}"
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
        append_random_mockery(&mut message, &COMPLETE_MISS_COMMENTARY);
        let _ = defer.await;
        let _ = ctx.reply(message).await;

        return Ok(());
    }

    if damage_dies > 0 {
        let query = ParsedRollQuery::new(damage_dies.into(), None, None, None);
        let damage_roll_result = query.execute();

        let maybe_crit = if accuracy_roll_result.is_critical_hit {
            "(+CRIT)"
        } else {
            ""
        };

        message.push_str(&format!(
            "**Damage roll**: {} {maybe_crit}\n",
            damage_roll_result.message.replace("\n", " – ")
        ));

        if damage_roll_result.success_count == 0 {
            append_random_mockery(&mut message, &ZERO_DAMAGE_COMMENTARY);
        } else if damage_roll_result.success_count == damage_dies {
            append_random_mockery(&mut message, &ALL_HIT_COMMENTARY);
        }
    }

    if let Some(status_effect_dies) = status_effect_dies {
        let query = ParsedRollQuery::new(status_effect_dies.into(), None, None, None);
        let status_roll_result = query.execute();

        message.push_str(&format!(
            "**Status Effect roll**: {}\n",
            status_roll_result.message.replace("\n", " – ")
        ));
    }

    if let Some(status_effect_dies) = status_effect_dies_2 {
        let query = ParsedRollQuery::new(status_effect_dies.into(), None, None, None);
        let status_roll_result = query.execute();

        message.push_str(&format!(
            "**Status Effect #2 roll**: {}\n",
            status_roll_result.message.replace("\n", " – ")
        ));
    }

    let _ = defer.await;
    let _ = ctx.reply(message).await;
    Ok(())
}

fn append_random_mockery(message: &mut String, from: &'static [&'static str]) {
    let mut rng = rand::rng();
    let mockery = from.choose(&mut rng).expect("This should never be empty!");
    message.push_str(&format!("*{}*\n", mockery));
}

const ALL_HIT_COMMENTARY: [&str; 51] = [
    "Boom! Right on the money!",
    "That hit like a truck full of determination!",
    "No mercy, no misses — just pure power!",
    "Everything connected — and then some!",
    "An absolute textbook strike!",
    "That one echoed through the arena!",
    "A flawless execution!",
    "That’s gotta sting!",
    "Did you see the form on that hit?",
    "Maximum efficiency, maximum pain!",
    "All power, all precision!",
    "A beautiful example of doing everything right!",
    "That hit was art. Violent art.",
    "A picture-perfect strike!",
    "Critical technique, zero hesitation!",
    "Direct hit! Devastation confirmed!",
    "No wasted motion — just results.",
    "That one’s going in the highlight reel!",
    "An absolute home run!",
    "Full marks for pain delivery!",
    "Bang! Everything hit like a dream.",
    "They brought the pain. And then brought more.",
    "One clean strike — total devastation!",
    "Damage dice? More like damage destiny.",
    "That opponent felt all of it.",
    "When they land it, they really land it.",
    "If that’s not power, I don’t know what is.",
    "They hit with the force of storytelling momentum!",
    "No hesitation, no forgiveness.",
    "Who gave them permission to hit that hard?",
    "It was as if the universe wanted that hit to land.",
    "Their opponent is questioning life choices.",
    "Absolutely obliterated!",
    "That was just... disrespectfully effective.",
    "A demolition in one strike!",
    "Ten out of ten — from every angle.",
    "Ruthless. Precise. Beautiful.",
    "A jaw-dropper of a hit!",
    "You could hear that from the next region.",
    "An attack worthy of a championship belt.",
    "That one broke the sound barrier.",
    "Delivered with love... and incredible force.",
    "Total commitment. Total damage.",
    "They didn't just hit — they made a statement.",
    "They made the battlefield their canvas.",
    "That was the move they practiced all week.",
    "Their opponent’s soul left the building.",
    "One hit. All heart.",
    "Perfect timing, perfect strike.",
    "They hit the sweet spot so hard it cracked.",
    "Someone’s going to feel that tomorrow — and next week.",
];

const ZERO_DAMAGE_COMMENTARY: [&str; 51] = [
    "That hit looked way cooler than it felt.",
    "Made contact, did nothing.",
    "It's like a handshake. With dramatic flair.",
    "A friendly reminder, not a threat.",
    "It technically hit, yes.",
    "That was a very polite attack.",
    "Their opponent barely blinked.",
    "All sizzle, no steak.",
    "Damage output: theatrical, but ineffective.",
    "Hit confirmed. Impact denied.",
    "That did more to the air than the target.",
    "That was more of a statement than an attack.",
    "They grazed their opponent’s confidence, maybe.",
    "It was like a warning shot. But worse.",
    "They touched victory... and immediately let go.",
    "They dealt zero damage and zero dignity.",
    "That hit will echo forever... in embarrassment.",
    "Their opponent is confused but unharmed.",
    "No damage, but so much effort!",
    "Could’ve been something. Wasn’t.",
    "That was an emotional tap, not a real one.",
    "I think their opponent just shrugged it off.",
    "Well, they made their point. Sort of.",
    "Like being hit with a rolled-up napkin.",
    "Even the referee winced... from awkwardness.",
    "That hit was legally classified as 'gentle'.",
    "Perfect aim. Pathetic damage.",
    "Technically correct — the worst kind of correct.",
    "They hit them like a wet leaf.",
    "A dramatic flourish with no follow-through.",
    "They tried to hurt them. The universe said no.",
    "One strike. Zero impact.",
    "All that effort for a friendly tap.",
    "That did more to their own confidence.",
    "Even the target looked surprised to be untouched.",
    "Like punching a memory foam mattress.",
    "That was basically a high five.",
    "They hit with the intensity of a lullaby.",
    "The air moved more than the target did.",
    "Just a soft little nudge of failure.",
    "That hit landed... emotionally.",
    "A glancing blow on the opponent's patience.",
    "Like blowing a raspberry in combat.",
    "Definitely not worth the energy cost.",
    "Even the damage dice are embarrassed.",
    "At least they practiced the motion.",
    "If disappointment were a stat, it's maxed out.",
    "A good swing. A better letdown.",
    "The target barely noticed the breeze.",
    "Maybe next time try hurting them?",
    "That hit was the nicest thing they’ve ever done in battle.",
];

const COMPLETE_MISS_COMMENTARY: [&str; 50] = [
    "Oof — not even close!",
    "They just attacked the idea of the target.",
    "A masterclass in missing.",
    "The only thing they hit was their pride.",
    "That was... aspirational.",
    "Air got obliterated. The opponent? Not so much.",
    "Someone’s going to pretend that was intentional.",
    "That target was in a different timezone.",
    "Their opponent didn’t even blink.",
    "Even gravity dodged that one.",
    "That was a warning shot. Hopefully.",
    "They attacked the memory of the opponent.",
    "An aggressive display of spatial misunderstanding.",
    "A truly strategic use of failure.",
    "New Olympic event: synchronized whiffing!",
    "If you don’t hit, you can’t miss... oh wait.",
    "They aimed with their soul. Unfortunately.",
    "The ghost of that move might haunt someone someday.",
    "Nowhere near. But very confidently so.",
    "A flawless display of bad luck.",
    "The floor took a lot of damage. The target? Not so much.",
    "That was like yelling into the void — but with effort.",
    "Worthy of mockery!",
    "It was an attack. It just wasn’t a good one.",
    "They've mastered the art of missing with style.",
    "A swing, a spin, a total fail!",
    "Was that a feint? No? Just a miss? Okay.",
    "They had one job.",
    "Even their shadow is shaking its head.",
    "I've seen better aim from a fellow Magikarp!",
    "That was less an attack, more a motion.",
    "Wind resistance: 1. Accuracy: 0.",
    "They missed and emotionally damaged the audience.",
    "That strike is now in orbit.",
    "Someone just facepalmed.",
    "An elegant miss, if nothing else.",
    "The attack landed in an alternate dimension.",
    "They used Foresight... but forgot to hit.",
    "That wasn’t a move — that was wishful thinking.",
    "They missed so hard it created a gust.",
    "Nobody knows what they were aiming at.",
    "Their Pokémon looked proud... but why?",
    "The opponent politely applauded the attempt.",
    "If you miss with flair, does it count?",
    "The miss was so clean it deserves a replay.",
    "They wrote a love letter to failure.",
    "That attack had commitment, just no accuracy.",
    "They boldly struck nothing at all.",
    "It looked cool. Shame about the result.",
    "It missed, but the drama was top tier.",
];
