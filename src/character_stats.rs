use crate::emoji;
use crate::game_data::pokemon::{Pokemon, PokemonStat};
use std::cmp::Ordering;

enum CharacterStatType {
    Combat { base_hp: u8 },
    Social,
}

pub struct GenericCharacterStats {
    kind: CharacterStatType,
    strength_or_tough: CharacterStat,
    dexterity_or_cool: CharacterStat,
    vitality_or_beauty: CharacterStat,
    special_or_cute: CharacterStat,
    insight_or_clever: CharacterStat,
}

struct CharacterStat {
    current: i64,
    min: i64,
    max: i64,
}

impl CharacterStat {
    pub fn from_poke(current: i64, stat: &PokemonStat) -> Self {
        Self {
            current,
            min: stat.min as i64,
            max: stat.max as i64,
        }
    }
    pub fn from_social(current: i64) -> Self {
        Self {
            current,
            min: 1,
            max: 5,
        }
    }
    pub fn invested_points(&self) -> i64 {
        // TODO: Take Limit Breaking into account
        self.current - self.min
    }
}

const COMBAT_PADDING: usize = 10;
const SOCIAL_PADDING: usize = 7;

impl GenericCharacterStats {
    pub fn from_combat(
        pokemon: &Pokemon,
        strength: i64,
        dexterity: i64,
        vitality: i64,
        special: i64,
        insight: i64,
    ) -> Self {
        GenericCharacterStats {
            kind: CharacterStatType::Combat {
                base_hp: pokemon.base_hp,
            },
            strength_or_tough: CharacterStat::from_poke(strength, &pokemon.strength),
            dexterity_or_cool: CharacterStat::from_poke(dexterity, &pokemon.dexterity),
            vitality_or_beauty: CharacterStat::from_poke(vitality, &pokemon.vitality),
            special_or_cute: CharacterStat::from_poke(special, &pokemon.special),
            insight_or_clever: CharacterStat::from_poke(insight, &pokemon.insight),
        }
    }

    pub fn from_social(tough: i64, cool: i64, beauty: i64, cute: i64, clever: i64) -> Self {
        GenericCharacterStats {
            kind: CharacterStatType::Social,
            strength_or_tough: CharacterStat::from_social(tough),
            dexterity_or_cool: CharacterStat::from_social(cool),
            vitality_or_beauty: CharacterStat::from_social(beauty),
            special_or_cute: CharacterStat::from_social(cute),
            insight_or_clever: CharacterStat::from_social(clever),
        }
    }

    pub fn build_string(&self) -> String {
        match self.kind {
            CharacterStatType::Combat { base_hp } => {
                format!(
                    "\
HP: {}
Willpower: {}

{}
Defense: {}
Special Defense: {}
",
                    (base_hp as i64 + self.vitality_or_beauty.current) * 2,
                    self.insight_or_clever.current + 2,
                    self.build_stat_block(),
                    (self.vitality_or_beauty.current as f32 * 0.5).ceil(),
                    (self.insight_or_clever.current as f32 * 0.5).ceil()
                )
            }
            CharacterStatType::Social => self.build_stat_block(),
        }
    }

    fn build_stat_block(&self) -> String {
        match self.kind {
            CharacterStatType::Combat { .. } => {
                format!(
                    "{}{}{}{}{}",
                    GenericCharacterStats::build_stat_row(
                        "Strength",
                        &self.strength_or_tough,
                        COMBAT_PADDING
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Dexterity",
                        &self.dexterity_or_cool,
                        COMBAT_PADDING
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Vitality",
                        &self.vitality_or_beauty,
                        COMBAT_PADDING
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Special",
                        &self.special_or_cute,
                        COMBAT_PADDING
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Insight",
                        &self.insight_or_clever,
                        COMBAT_PADDING
                    ),
                )
            }
            CharacterStatType::Social => {
                format!(
                    "{}{}{}{}{}",
                    GenericCharacterStats::build_stat_row(
                        "Tough",
                        &self.strength_or_tough,
                        SOCIAL_PADDING
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Cool",
                        &self.dexterity_or_cool,
                        SOCIAL_PADDING
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Beauty",
                        &self.vitality_or_beauty,
                        SOCIAL_PADDING
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Cute",
                        &self.special_or_cute,
                        SOCIAL_PADDING
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Clever",
                        &self.insight_or_clever,
                        SOCIAL_PADDING
                    ),
                )
            }
        }
    }

    fn build_stat_row(name: &str, stat: &CharacterStat, padding: usize) -> String {
        let mut result = String::new();
        result.push_str(&format!(
            "{:<padding$} {} |",
            format!("{}:", name),
            stat.current,
            padding = padding
        ));

        match stat.current.cmp(&stat.max) {
            Ordering::Less => {
                for i in 0..stat.max {
                    if i < stat.current {
                        result.push(emoji::DOT_FILLED);
                    } else {
                        result.push(emoji::DOT_EMPTY);
                    }
                }
            }
            Ordering::Equal => {
                for _ in 0..stat.current {
                    result.push(emoji::DOT_FILLED);
                }
            }
            Ordering::Greater => {
                for i in 0..stat.current {
                    if i < stat.max {
                        result.push(emoji::DOT_FILLED);
                    } else {
                        result.push(emoji::DOT_OVERCHARGED);
                    }
                }
            }
        }

        result.push('\n');
        result
    }

    pub fn calculate_invested_stat_points(&self) -> i64 {
        // TODO: Take Limit Breaking into account
        self.strength_or_tough.invested_points()
            + self.insight_or_clever.invested_points()
            + self.special_or_cute.invested_points()
            + self.vitality_or_beauty.invested_points()
            + self.dexterity_or_cool.invested_points()
    }
}
