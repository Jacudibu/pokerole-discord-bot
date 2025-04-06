use crate::shared::emoji;
use crate::shared::game_data::pokemon::{Pokemon, PokemonStat};
use std::cmp::Ordering;

enum CharacterStatType {
    Combat { base_hp: u8 },
    Social,
}

#[derive(Debug, Copy, Clone)]
pub enum SingleCharacterStatType {
    Strength,
    Dexterity,
    Vitality,
    Special,
    Insight,
    Tough,
    Cool,
    Beauty,
    Cute,
    Clever,
}

impl SingleCharacterStatType {
    pub fn is_combat_stat(&self) -> bool {
        match self {
            SingleCharacterStatType::Strength => true,
            SingleCharacterStatType::Dexterity => true,
            SingleCharacterStatType::Vitality => true,
            SingleCharacterStatType::Special => true,
            SingleCharacterStatType::Insight => true,
            SingleCharacterStatType::Tough => false,
            SingleCharacterStatType::Cool => false,
            SingleCharacterStatType::Beauty => false,
            SingleCharacterStatType::Cute => false,
            SingleCharacterStatType::Clever => false,
        }
    }
}

pub struct GenericCharacterStats {
    kind: CharacterStatType,
    strength_or_tough: CharacterStat,
    dexterity_or_cool: CharacterStat,
    vitality_or_beauty: CharacterStat,
    special_or_cute: CharacterStat,
    insight_or_clever: CharacterStat,
}

impl GenericCharacterStats {
    pub fn get(&self, stat: SingleCharacterStatType) -> &CharacterStat {
        match stat {
            SingleCharacterStatType::Strength | SingleCharacterStatType::Tough => {
                &self.strength_or_tough
            }
            SingleCharacterStatType::Dexterity | SingleCharacterStatType::Cool => {
                &self.dexterity_or_cool
            }
            SingleCharacterStatType::Vitality | SingleCharacterStatType::Beauty => {
                &self.vitality_or_beauty
            }
            SingleCharacterStatType::Special | SingleCharacterStatType::Cute => {
                &self.special_or_cute
            }
            SingleCharacterStatType::Insight | SingleCharacterStatType::Clever => {
                &self.insight_or_clever
            }
        }
    }
}

pub struct CharacterStat {
    pub current: i64,
    pub currently_set_on_character: i64,
    pub species_min: i64,
    pub species_max: i64,
}

impl CharacterStat {
    pub fn new(min: i64, current: i64, species_min: i64, species_max: i64) -> Self {
        Self {
            current,
            species_min,
            species_max,
            currently_set_on_character: min,
        }
    }
    pub fn from_poke(current: i64, stat: &PokemonStat) -> Self {
        Self {
            current,
            currently_set_on_character: current,
            species_min: stat.min as i64,
            species_max: stat.max as i64,
        }
    }
    pub fn from_poke_with_min(current: i64, min: i64, stat: &PokemonStat) -> Self {
        Self {
            current,
            currently_set_on_character: min,
            species_min: stat.min as i64,
            species_max: stat.max as i64,
        }
    }
    pub fn from_social(current: i64) -> Self {
        Self {
            current,
            currently_set_on_character: current,
            species_min: 1,
            species_max: 5,
        }
    }
    pub fn invested_points(&self) -> i64 {
        self.current - self.species_min
    }
    pub fn count_limit_breaks(&self) -> i64 {
        if self.current > self.species_max {
            self.current - self.species_max
        } else {
            0
        }
    }
    pub fn is_at_or_above_max(&self) -> bool {
        self.current >= self.species_max
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

    #[allow(clippy::too_many_arguments)]
    pub fn from_combat_with_current_min(
        pokemon: &Pokemon,
        strength_current: i64,
        strength_min: i64,
        dexterity_current: i64,
        dexterity_min: i64,
        vitality_current: i64,
        vitality_min: i64,
        special_current: i64,
        special_min: i64,
        insight_current: i64,
        insight_min: i64,
    ) -> Self {
        GenericCharacterStats {
            kind: CharacterStatType::Combat {
                base_hp: pokemon.base_hp,
            },
            strength_or_tough: CharacterStat::from_poke_with_min(
                strength_current,
                strength_min,
                &pokemon.strength,
            ),
            dexterity_or_cool: CharacterStat::from_poke_with_min(
                dexterity_current,
                dexterity_min,
                &pokemon.dexterity,
            ),
            vitality_or_beauty: CharacterStat::from_poke_with_min(
                vitality_current,
                vitality_min,
                &pokemon.vitality,
            ),
            special_or_cute: CharacterStat::from_poke_with_min(
                special_current,
                special_min,
                &pokemon.special,
            ),
            insight_or_clever: CharacterStat::from_poke_with_min(
                insight_current,
                insight_min,
                &pokemon.insight,
            ),
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

    #[allow(clippy::too_many_arguments)]
    pub fn from_social_with_current_min(
        tough_current: i64,
        tough_min: i64,
        cool_current: i64,
        cool_min: i64,
        beauty_current: i64,
        beauty_min: i64,
        cute_current: i64,
        cute_min: i64,
        clever_current: i64,
        clever_min: i64,
    ) -> Self {
        GenericCharacterStats {
            kind: CharacterStatType::Social,
            strength_or_tough: CharacterStat::new(tough_min, tough_current, 1, 5),
            dexterity_or_cool: CharacterStat::new(cool_min, cool_current, 1, 5),
            vitality_or_beauty: CharacterStat::new(beauty_min, beauty_current, 1, 5),
            special_or_cute: CharacterStat::new(cute_min, cute_current, 1, 5),
            insight_or_clever: CharacterStat::new(clever_min, clever_current, 1, 5),
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
Active Move Limit: {}
",
                    (base_hp as i64 + self.vitality_or_beauty.current) * 2,
                    self.insight_or_clever.current + 2,
                    self.build_stat_block(),
                    (self.vitality_or_beauty.current as f32 * 0.5).ceil(),
                    (self.insight_or_clever.current as f32 * 0.5).ceil(),
                    self.insight_or_clever.current + 2,
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

        match stat.current.cmp(&stat.species_max) {
            Ordering::Less => {
                for i in 0..stat.species_max {
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
                    if i < stat.species_max {
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
        let limit_breaks = self.count_limit_breaks();
        let used_extra_points_for_limit_breaks = if limit_breaks > 0 {
            (limit_breaks * limit_breaks + limit_breaks) / 2
        } else {
            0
        };

        used_extra_points_for_limit_breaks
            + self.strength_or_tough.invested_points()
            + self.insight_or_clever.invested_points()
            + self.special_or_cute.invested_points()
            + self.vitality_or_beauty.invested_points()
            + self.dexterity_or_cool.invested_points()
    }

    pub fn count_limit_breaks(&self) -> i64 {
        self.strength_or_tough.count_limit_breaks()
            + self.insight_or_clever.count_limit_breaks()
            + self.special_or_cute.count_limit_breaks()
            + self.vitality_or_beauty.count_limit_breaks()
            + self.dexterity_or_cool.count_limit_breaks()
    }

    pub fn is_any_stat_at_or_above_max(&self) -> bool {
        self.strength_or_tough.is_at_or_above_max()
            || self.insight_or_clever.is_at_or_above_max()
            || self.special_or_cute.is_at_or_above_max()
            || self.vitality_or_beauty.is_at_or_above_max()
            || self.dexterity_or_cool.is_at_or_above_max()
    }
}
