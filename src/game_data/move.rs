use std::str::FromStr;
use log::error;
use crate::game_data::parser::custom_data::custom_move::CustomMove;
use crate::game_data::pokerole_data::raw_move::RawPokeroleMove;
use crate::enums::{CombatOrSocialStat, HappinessDamageModifier, MoveCategory, MoveType, Stat};

pub struct Move {
    pub name: String,
    pub typing: MoveType,
    pub power: u8,
    pub damage1: Option<Stat>,
    pub happiness_damage: Option<HappinessDamageModifier>,
    pub accuracy1: Option<CombatOrSocialStat>,
    pub accuracy2: Option<CombatOrSocialStat>,
    pub target: String,
    pub effect: Option<String>,
    pub description: Option<String>,
    //pub attributes - includes stuff like never_fail: bool. But that's already written in effect.
    //pub added_effects - includes stuff like stat changes. PARSEABLE stat changes! But they are already written in effect.
    pub category: MoveCategory,
}

fn replace_effect_string(raw: &str) -> Option<String> {
    if raw == "-" {
        return None;
    }

    Some(raw.replace("1 lethal", "1 Wound")
            .replace("1 Lethal", "1 Wound")
            .replace("cure Lethal", "cure Wound")
            .replace("Lethal", "Inflicts Wounds")
            .replace("Basic Heal", "Heal 5 HP")
            .replace("Complete Heal", "Heal 10 HP")
            .replace("Full Heal", "Heal 10 HP")
    )
}

impl Move {
    pub(in crate::game_data) fn new(raw: &RawPokeroleMove) -> Self {
        Move {
            name: raw.name.clone(),
            typing: raw.r#type,
            power: raw.power,
            damage1: Move::parse_damage1(raw.damage1.clone()),
            happiness_damage: Move::parse_happiness_damage(raw.damage2.clone()),
            accuracy1: Move::parse_accuracy(raw.accuracy1.clone()),
            accuracy2: Move::parse_accuracy(raw.accuracy2.clone()),
            target: raw.target.clone(),
            effect: replace_effect_string(&raw.effect),
            description: Move::parse_description(raw.description.clone()),
            category: raw.category
        }
    }

    pub(in crate::game_data) fn from_custom(raw: &CustomMove) -> Self {
        Move {
            name: raw.name.clone(),
            typing: raw.r#type,
            power: raw.power,
            damage1: Move::parse_damage1(raw.damage.clone()),
            happiness_damage: None,
            accuracy1: Move::parse_accuracy(raw.accuracy.clone()),
            accuracy2: Some(CombatOrSocialStat::Rank),
            target: raw.target.clone(),
            effect: replace_effect_string(&raw.effect),
            description: Move::parse_description(raw.description.clone()),
            category: raw.category
        }
    }

    fn parse_description(raw: String) -> Option<String> {
        if raw.is_empty() {
            return None;
        }

        Some(raw)
    }

    fn parse_damage1(raw: String) -> Option<Stat> {
        if raw.is_empty() {
            return None;
        }

        return match Stat::from_str(&raw) {
            Ok(result) => Some(result),
            Err(_) => {
                match raw.as_str() {
                    "Strength/special" => Some(Stat::StrengthOrSpecial),
                    "Same as the copied move" => Some(Stat::Copy),
                    _ => {
                        error!("Cannot parse damage modifier: {}", &raw);
                        None
                    }
                }
            }
        }
    }

    fn parse_happiness_damage(raw: String) -> Option<HappinessDamageModifier> {
        if raw.is_empty() {
            return None;
        }

        return match raw.as_str() {
            "Happiness" => Some(HappinessDamageModifier::Happiness),
            "Missing happiness" => Some(HappinessDamageModifier::MissingHappiness),
            _ => {
                error!("Cannot parse happiness damage modifier: {}", &raw);
                None
            }
        }
    }

    fn parse_accuracy(raw: String) -> Option<CombatOrSocialStat> {
        if raw.is_empty() {
            return None;
        }

        return match CombatOrSocialStat::from_str(&raw) {
            Ok(result) => Some(result),
            Err(_) => {
                match raw.as_str() {
                    "Missing beauty" => Some(CombatOrSocialStat::MissingBeauty),
                    "BRAWL/CHANNEL" => Some(CombatOrSocialStat::BrawlOrChannel),
                    "Tough/cute" => Some(CombatOrSocialStat::ToughOrCute),
                    "Same as the copied move" => Some(CombatOrSocialStat::Copied),
                    "BRAWL" => Some(CombatOrSocialStat::Brawl),
                    "PERFORM" => Some(CombatOrSocialStat::Perform),
                    "ALLURE" => Some(CombatOrSocialStat::Allure),
                    _ => {
                        error!("Cannot parse accuracy modifier: {}", &raw);
                        None
                    }
                }
            }
        }
    }

    pub(crate) fn build_string(&self) -> String {
        let mut result : String = std::format!("### {}\n", &self.name);
        if let Some(description) = &self.description {
            result.push('*');
            result.push_str(description);
            result.push_str("*\n");
        }

        result.push_str("**Type**: ");
        if self.typing == MoveType::Typeless {
            result.push_str("None");
        } else {
            result.push_str(std::format!("{}", self.typing).as_str());
        }
        if self.name != "Metronome" {
            result.push_str(" — **");
            result.push_str(std::format!("{}", self.category).as_str());
            result.push_str("**\n");

            result.push_str("**Target**: ");
            result.push_str(self.target.to_string().as_str());
            result.push('\n');

            if self.damage1.is_some() || self.happiness_damage.is_some() || self.power > 0 {
                result.push_str("**Damage Dice**: ");
                if let Some(stat) = self.damage1 {
                    result.push_str(std::format!("{}", stat).as_str());
                    result.push_str(" + ");
                }
                if let Some(stat) = self.happiness_damage {
                    result.push_str(std::format!("{}", stat).as_str());
                    result.push_str(" + ");
                }
                result.push_str(&std::format!("{}\n", self.power));
            }

            result.push_str("**Accuracy Dice**: ");
            if let Some(stat) = self.accuracy1 {
                result.push_str(std::format!("{}", stat).as_str());

                if self.accuracy2.is_some() {
                    result.push_str(" + Rank");
                }
            }
        }

        result.push('\n');
        if let Some(effect) = &self.effect {
            result.push_str("**Effect**: ");
            result.push_str(effect);
        }

        result
    }
}
