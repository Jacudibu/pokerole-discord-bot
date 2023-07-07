use crate::emoji;
use serde::Deserialize;
use sqlx::database::{HasArguments, HasValueRef};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::Sqlite;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum_macros::{EnumIter, EnumString, FromRepr};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, EnumString, Hash, EnumIter)]
pub enum PokemonType {
    Normal,
    Fighting,
    Flying,
    Poison,
    Ground,
    Rock,
    Bug,
    Ghost,
    Steel,
    Fire,
    Water,
    Grass,
    Electric,
    Psychic,
    Ice,
    Dragon,
    Dark,
    Fairy,
    Shadow,
}

impl fmt::Display for PokemonType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PokemonType::Normal => "<:type_normal:1118590014931095662> Normal",
            PokemonType::Fighting => "<:type_fighting:1118590013194649730> Fighting",
            PokemonType::Flying => "<:type_flying:1118590010359283773> Flying",
            PokemonType::Poison => "<:type_poison:1118590008778047529> Poison",
            PokemonType::Ground => "<:type_ground:1118590006081114325> Ground",
            PokemonType::Rock => "<:type_rock:1118590005082861820> Rock",
            PokemonType::Bug => "<:type_bug:1118594892566908959> Bug",
            PokemonType::Ghost => "<:type_ghost:1118594890461368350> Ghost",
            PokemonType::Steel => "<:type_steel:1118594889131765821> Steel",
            PokemonType::Fire => "<:type_fire:1118594887399514145> Fire",
            PokemonType::Water => "<:type_water:1118594885344297062> Water",
            PokemonType::Grass => "<:type_grass:1118594883754664107> Grass",
            PokemonType::Electric => "<:type_electric:1118594871272415243> Electric",
            PokemonType::Psychic => "<:type_psychic:1118594873755435009> Psychic",
            PokemonType::Ice => "<:type_ice:1118594875085041825> Ice",
            PokemonType::Dragon => "<:type_dragon:1118594876444000357> Dragon",
            PokemonType::Dark => "<:type_dark:1118594879195447387> Dark",
            PokemonType::Fairy => "<:type_fairy:1118594881368100894> Fairy",
            PokemonType::Shadow => "Shadow",
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum MoveType {
    Normal,
    Fighting,
    Flying,
    Poison,
    Ground,
    Rock,
    Bug,
    Ghost,
    Steel,
    Fire,
    Water,
    Grass,
    Electric,
    Psychic,
    Ice,
    Dragon,
    Dark,
    Fairy,
    Any,
    Typeless,
}

impl fmt::Display for MoveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MoveType::Normal => "<:type_normal:1118590014931095662> Normal",
            MoveType::Fighting => "<:type_fighting:1118590013194649730> Fighting",
            MoveType::Flying => "<:type_flying:1118590010359283773> Flying",
            MoveType::Poison => "<:type_poison:1118590008778047529> Poison",
            MoveType::Ground => "<:type_ground:1118590006081114325> Ground",
            MoveType::Rock => "<:type_rock:1118590005082861820> Rock",
            MoveType::Bug => "<:type_bug:1118594892566908959> Bug",
            MoveType::Ghost => "<:type_ghost:1118594890461368350> Ghost",
            MoveType::Steel => "<:type_steel:1118594889131765821> Steel",
            MoveType::Fire => "<:type_fire:1118594887399514145> Fire",
            MoveType::Water => "<:type_water:1118594885344297062> Water",
            MoveType::Grass => "<:type_grass:1118594883754664107> Grass",
            MoveType::Electric => "<:type_electric:1118594871272415243> Electric",
            MoveType::Psychic => "<:type_psychic:1118594873755435009> Psychic",
            MoveType::Ice => "<:type_ice:1118594875085041825> Ice",
            MoveType::Dragon => "<:type_dragon:1118594876444000357> Dragon",
            MoveType::Dark => "<:type_dark:1118594879195447387> Dark",
            MoveType::Fairy => "<:type_fairy:1118594881368100894> Fairy",
            MoveType::Any => "Any",
            MoveType::Typeless => "Typeless",
        })
    }
}

impl MoveType {
    pub fn has_stab(&self, poke_type: &Option<PokemonType>) -> bool {
        if let Some(poke_type) = poke_type {
            match poke_type {
                PokemonType::Normal => self == &MoveType::Normal,
                PokemonType::Fighting => self == &MoveType::Fighting,
                PokemonType::Flying => self == &MoveType::Flying,
                PokemonType::Poison => self == &MoveType::Poison,
                PokemonType::Ground => self == &MoveType::Ground,
                PokemonType::Rock => self == &MoveType::Rock,
                PokemonType::Bug => self == &MoveType::Bug,
                PokemonType::Ghost => self == &MoveType::Ghost,
                PokemonType::Steel => self == &MoveType::Steel,
                PokemonType::Fire => self == &MoveType::Fire,
                PokemonType::Water => self == &MoveType::Water,
                PokemonType::Grass => self == &MoveType::Grass,
                PokemonType::Electric => self == &MoveType::Electric,
                PokemonType::Psychic => self == &MoveType::Psychic,
                PokemonType::Ice => self == &MoveType::Ice,
                PokemonType::Dragon => self == &MoveType::Dragon,
                PokemonType::Dark => self == &MoveType::Dark,
                PokemonType::Fairy => self == &MoveType::Fairy,
                PokemonType::Shadow => false,
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum MoveCategory {
    Physical,
    Special,
    #[serde(rename = "Physical/special")]
    /// Only used for struggle and tera blast
    PhysicalOrSpecial,
    Support,
}

impl fmt::Display for MoveCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MoveCategory::Physical => "<:move_physical:1118637143267487925> Physical",
            MoveCategory::Special => "<:move_special:1118637141862404217> Special",
            MoveCategory::PhysicalOrSpecial => "<:move_physical:1118637143267487925> Physical / <:move_special:1118637141862404217> Special",
            MoveCategory::Support => "<:move_status:1118637139589091338> Support"
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Deserialize)]
pub enum MysteryDungeonRank {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

impl MysteryDungeonRank {
    pub fn die_count(&self) -> u8 {
        match self {
            MysteryDungeonRank::Bronze => 1,
            MysteryDungeonRank::Silver => 2,
            MysteryDungeonRank::Gold => 3,
            MysteryDungeonRank::Platinum => 4,
            MysteryDungeonRank::Diamond => 5,
        }
    }

    pub fn from_level(level: u8) -> Self {
        match level {
            1 => MysteryDungeonRank::Bronze,
            2..=3 => MysteryDungeonRank::Silver,
            4..=7 => MysteryDungeonRank::Gold,
            8..=15 => MysteryDungeonRank::Platinum,
            _ => MysteryDungeonRank::Diamond,
        }
    }

    pub fn emoji_string(&self) -> &str {
        match self {
            MysteryDungeonRank::Bronze => emoji::RANK_BRONZE,
            MysteryDungeonRank::Silver => emoji::RANK_SILVER,
            MysteryDungeonRank::Gold => emoji::RANK_GOLD,
            MysteryDungeonRank::Platinum => emoji::RANK_PLATINUM,
            MysteryDungeonRank::Diamond => emoji::RANK_DIAMOND,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
pub enum RegionalVariant {
    Alola,
    Galar,
    Hisui,
    Paldea,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, EnumString)]
pub enum Stat {
    Strength,
    Dexterity,
    Vitality,
    Special,
    Insight,
    /// Used for Struggle
    StrengthOrSpecial,
    /// Used for Copycat
    Copy,
    /// Used for Fixed Damage moves
    Rank,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SocialStat {
    Tough,
    Cool,
    Beauty,
    Clever,
    Cute,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
pub enum Gender {
    Genderless,
    Male,
    Female,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Gender::Genderless => "–",
            Gender::Male => "M",
            Gender::Female => "F",
        })
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Stat::Strength => "Strength",
            Stat::Dexterity => "Dexterity",
            Stat::Vitality => "Vitality",
            Stat::Special => "Special",
            Stat::Insight => "Insight",
            Stat::StrengthOrSpecial => "Strength / Special",
            Stat::Copy => "Copy",
            Stat::Rank => "Rank",
        })
    }
}

#[derive(Debug, Clone, Copy, EnumString)]
pub enum CombatOrSocialStat {
    Strength,
    Dexterity,
    Vitality,
    Special,
    Insight,
    Tough,
    Cool,
    Beauty,
    Clever,
    Cute,
    Brawl,
    Channel,
    Clash,
    Evasion,
    Alert,
    Athletic,
    Nature,
    Stealth,
    Allure,
    Etiquette,
    Intimidate,
    Perform,
    Will,
    Copied,
    ToughOrCute,
    MissingBeauty,
    BrawlOrChannel,
    Varies,
    Medicine,
    Empathy,
    Rank,
}

impl fmt::Display for CombatOrSocialStat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CombatOrSocialStat::Strength => "Strength",
            CombatOrSocialStat::Dexterity => "Dexterity",
            CombatOrSocialStat::Vitality => "Vitality",
            CombatOrSocialStat::Special => "Special",
            CombatOrSocialStat::Insight => "Insight",
            CombatOrSocialStat::Tough => "Tough",
            CombatOrSocialStat::Cool => "Cool",
            CombatOrSocialStat::Beauty => "Beauty",
            CombatOrSocialStat::Clever => "Clever",
            CombatOrSocialStat::Cute => "Cute",
            CombatOrSocialStat::Brawl => "Brawl",
            CombatOrSocialStat::Channel => "Channel",
            CombatOrSocialStat::Clash => "Clash",
            CombatOrSocialStat::Evasion => "Evasion",
            CombatOrSocialStat::Alert => "Alert",
            CombatOrSocialStat::Athletic => "Athletic",
            CombatOrSocialStat::Nature => "Nature",
            CombatOrSocialStat::Stealth => "Stealth",
            CombatOrSocialStat::Allure => "Allure",
            CombatOrSocialStat::Etiquette => "Etiquette",
            CombatOrSocialStat::Intimidate => "Intimidate",
            CombatOrSocialStat::Perform => "Perform",
            CombatOrSocialStat::Will => "Will",
            CombatOrSocialStat::Copied => "Copied",
            CombatOrSocialStat::ToughOrCute => "Tough / Cute",
            CombatOrSocialStat::MissingBeauty => "5 - Beauty",
            CombatOrSocialStat::BrawlOrChannel => "Brawl / Channel",
            CombatOrSocialStat::Varies => "Varies",
            CombatOrSocialStat::Medicine => "Medicine",
            CombatOrSocialStat::Empathy => "Empathy",
            CombatOrSocialStat::Rank => "Rank",
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HappinessDamageModifier {
    Happiness,
    MissingHappiness,
}

impl fmt::Display for HappinessDamageModifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            HappinessDamageModifier::Happiness => "Happiness",
            HappinessDamageModifier::MissingHappiness => "Missing Happiness",
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialOrd, PartialEq)]
pub enum PokemonGeneration {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    PartialOrd,
    PartialEq,
    poise::ChoiceParameter,
    sqlx::Type,
    FromRepr,
)]
#[repr(i64)]
pub enum QuestParticipantSelectionMechanism {
    #[name = "First Come First Serve"]
    FirstComeFirstServe = 1,
    Random = 2,
}
