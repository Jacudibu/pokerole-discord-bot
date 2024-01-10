use crate::emoji;
use crate::game_data::pokemon::Pokemon;

enum CharacterStatType {
    Combat { base_hp: u8 },
    Social,
}

struct GenericCharacterStats {
    kind: CharacterStatType,
    strength_or_tough: i64,
    strength_or_tough_max: i64,
    dexterity_or_cool: i64,
    dexterity_or_cool_max: i64,
    vitality_or_beauty: i64,
    vitality_or_beauty_max: i64,
    special_or_cute: i64,
    special_or_cute_max: i64,
    insight_or_clever: i64,
    insight_or_clever_max: i64,
}

impl GenericCharacterStats {
    pub fn from_combat(
        base_hp: u8,
        strength: i64,
        dexterity: i64,
        vitality: i64,
        special: i64,
        insight: i64,
        pokemon: &Pokemon,
    ) -> Self {
        GenericCharacterStats {
            kind: CharacterStatType::Combat { base_hp },
            strength_or_tough: strength,
            strength_or_tough_max: pokemon.strength.max as i64,
            dexterity_or_cool: dexterity,
            dexterity_or_cool_max: pokemon.dexterity.max as i64,
            vitality_or_beauty: vitality,
            vitality_or_beauty_max: pokemon.vitality.max as i64,
            special_or_cute: special,
            special_or_cute_max: pokemon.special.max as i64,
            insight_or_clever: insight,
            insight_or_clever_max: pokemon.insight.max as i64,
        }
    }

    pub fn from_social(tough: i64, cool: i64, beauty: i64, cute: i64, clever: i64) -> Self {
        GenericCharacterStats {
            kind: CharacterStatType::Social,
            strength_or_tough: tough,
            strength_or_tough_max: 5,
            dexterity_or_cool: cool,
            dexterity_or_cool_max: 5,
            vitality_or_beauty: beauty,
            vitality_or_beauty_max: 5,
            special_or_cute: cute,
            special_or_cute_max: 5,
            insight_or_clever: clever,
            insight_or_clever_max: 5,
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
                    (base_hp as i64 + self.vitality_or_beauty) * 2,
                    self.insight_or_clever + 2,
                    self.build_stat_block(),
                    (self.vitality_or_beauty as f32 * 0.5).ceil(),
                    (self.insight_or_clever as f32 * 0.5).ceil()
                )
            }
            CharacterStatType::Social => self.build_stat_block(),
        }
    }

    pub fn build_stat_block(&self) -> String {
        match self.kind {
            CharacterStatType::Combat { .. } => {
                format!(
                    "{}{}{}{}{}",
                    GenericCharacterStats::build_stat_row(
                        "Strength",
                        self.strength_or_tough,
                        self.strength_or_tough_max,
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Dexterity",
                        self.dexterity_or_cool,
                        self.dexterity_or_cool_max,
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Vitality",
                        self.vitality_or_beauty,
                        self.vitality_or_beauty_max,
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Special",
                        self.special_or_cute,
                        self.special_or_cute_max,
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Insight",
                        self.insight_or_clever,
                        self.insight_or_clever_max,
                    ),
                )
            }
            CharacterStatType::Social => {
                format!(
                    "{}{}{}{}{}",
                    GenericCharacterStats::build_stat_row(
                        "Tough",
                        self.strength_or_tough,
                        self.strength_or_tough_max,
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Cool",
                        self.dexterity_or_cool,
                        self.dexterity_or_cool_max,
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Beauty",
                        self.vitality_or_beauty,
                        self.vitality_or_beauty_max,
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Cute",
                        self.special_or_cute,
                        self.special_or_cute_max,
                    ),
                    GenericCharacterStats::build_stat_row(
                        "Clever",
                        self.insight_or_clever,
                        self.insight_or_clever_max,
                    ),
                )
            }
        }
    }

    fn build_stat_row(name: &str, value: i64, max: i64) -> String {
        let mut result = String::new();
        result.push_str(&format!("{}: {} |", name, value));

        if max == value {
            for _ in 0..value {
                result.push(emoji::DOT_FILLED);
            }
        } else if max > value {
            for i in 0..max {
                if i < value {
                    result.push(emoji::DOT_EMPTY);
                } else {
                    result.push(emoji::DOT_FILLED);
                }
            }
        } else if value > max {
            for i in 0..value {
                if i < max {
                    result.push(emoji::DOT_FILLED);
                } else {
                    result.push(emoji::DOT_OVERCHARGED);
                }
            }
        }

        result.push('\n');
        result
    }
}
