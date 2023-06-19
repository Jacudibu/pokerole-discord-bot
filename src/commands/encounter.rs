use rand::{Rng, thread_rng};
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use crate::commands::{Context, Error};
use crate::commands::autocompletion::autocomplete_pokemon;
use crate::game_data::pokemon::{Pokemon};
use crate::game_data::r#move::Move;
use crate::enums::{Stat, Gender, MysteryDungeonRank, PokemonType, SocialStat, CombatOrSocialStat};
use crate::game_data::GameData;
use crate::helpers;

/// Encounter some wild pokemon!
#[poise::command(slash_command)]
pub async fn encounter(
    ctx: Context<'_>,
    #[description = "Which pokemon?"]
    #[autocomplete = "autocomplete_pokemon"]
    pokemon: String,
    #[min = 1_u8]
    #[max = 100_u8]
    #[description = "Of which level?"]
    level: u8,
    #[min = 1_u8]
    #[max = 5_u8]
    #[description = "How many? Defaults to 1."]
    amount: Option<u8>
) -> Result<(), Error> {
    if let Some(pokemon) = ctx.data().game.pokemon.get(&pokemon.to_lowercase()) {
        for encounter in build_encounter(pokemon, level, amount) {
            for part in helpers::split_long_messages(encounter.build_string(pokemon, &ctx.data().game)) {
                ctx.say(part).await?;
            }
        }
    } else {
        ctx.send(|b| {
            b.content(std::format!("Unable to find a pokemon named **{}**, sorry! If that wasn't a typo, maybe it isn't implemented yet?", pokemon));
            b.ephemeral(true)
        }).await?;
    }

    Ok(())
}

fn build_encounter(pokemon: &Pokemon, level: u8, amount: Option<u8>) -> Vec<EncounterMon> {
    let mut result = Vec::new();
    for _ in 0..amount.unwrap_or(1) {
        result.push(EncounterMon::from_pokemon(pokemon, level));
    }

    result
}

#[derive(Debug)]
struct EncounterMon {
    pub name: String,
    pub gender: Gender,
    pub type1: PokemonType,
    pub type2: Option<PokemonType>,
    pub level: u8,
    pub rank: MysteryDungeonRank,
    pub ability: String,
    pub hp: u8,
    pub will: u8,
    pub strength: u8,
    pub dexterity: u8,
    pub vitality: u8,
    pub special: u8,
    pub insight: u8,
    pub tough: u8,
    pub cool: u8,
    pub beauty: u8,
    pub clever: u8,
    pub cute: u8,
    pub moves: Vec<String>,
}

impl EncounterMon {
    pub fn from_pokemon(pokemon: &Pokemon, level: u8) -> Self {
        let mut result = EncounterMon {
            name: pokemon.name.clone(),
            gender: EncounterMon::get_random_gender(pokemon),
            type1: pokemon.type1,
            type2: pokemon.type2,
            level,
            rank: EncounterMon::get_rank_from_level(level),
            ability: EncounterMon::get_random_ability(pokemon),
            hp: 0,
            will: 0,
            strength: pokemon.strength.min,
            dexterity: pokemon.dexterity.min,
            vitality: pokemon.vitality.min,
            special: pokemon.special.min,
            insight: pokemon.insight.min,
            tough: 1,
            cool: 1,
            beauty: 1,
            clever: 1,
            cute: 1,
            moves: Vec::new()
        };

        let mut rng = thread_rng();
        let all_stats = vec!(Stat::Strength, Stat::Vitality, Stat::Dexterity, Stat::Special, Stat::Insight);
        let mut non_maxed_stat_points = all_stats.clone();
        let mut remaining_stat_points = level + 3;
        let mut limit_break_count = 0;
        while remaining_stat_points > 0 {
            if let Some(mut stat) = non_maxed_stat_points.choose(&mut rng) {
                result.increase_stat(stat);

                if result.get_stat(stat) == pokemon.get_stat(stat).max {
                    let el_drop_o = *stat;
                    stat = &el_drop_o;
                    non_maxed_stat_points.retain(|x| x != stat);
                }
                remaining_stat_points -= 1;
            } else if remaining_stat_points > 2 + limit_break_count {
                result.increase_stat(all_stats.choose(&mut rng).unwrap());
                remaining_stat_points -= 2 + limit_break_count;
                limit_break_count += 1;
            } else {
                break;
            }
        }

        let mut non_maxed_social_stats = vec!(SocialStat::Tough, SocialStat::Cool, SocialStat::Beauty, SocialStat::Clever, SocialStat::Cute);
        let mut remaining_social_points = EncounterMon::calculate_social_points(&result.rank);
        while remaining_social_points > 0 {
            if let Some(mut stat) = non_maxed_social_stats.choose(&mut rng) {
                result.increase_social_stat(stat);

                if result.get_social_stat(stat) == 5 {
                    let el_drop_o = *stat;
                    stat = &el_drop_o;
                    non_maxed_social_stats.retain(|x| x != stat);
                }
            }

            remaining_social_points -= 1;
        }

        result.hp = (pokemon.base_hp + result.vitality) * 2;
        result.will = result.insight + 2;

        let available_moves = pokemon.moves.by_pokerole_rank
            .iter()
            .filter(|x| x.rank <= result.rank)
            .map(|x| x.name.clone());

        let move_count = result.insight + 2;
        result.moves = available_moves
            .choose_multiple(&mut thread_rng(), move_count as usize);

        result
    }

    fn calculate_social_points(rank: &MysteryDungeonRank) -> u8 {
        match rank {
            MysteryDungeonRank::Bronze => 4,
            MysteryDungeonRank::Silver => 4 + 2,
            MysteryDungeonRank::Gold => 4 + 4,
            MysteryDungeonRank::Platinum => 4 + 6,
            MysteryDungeonRank::Diamond => 4 + 8,
        }
    }

    fn get_random_gender(_pokemon: &Pokemon) -> Gender {
        // TODO: Use official gender ratio, lul.
        // Also, genderless mons.
        if thread_rng().gen_bool(0.5) {
            Gender::Male
        } else {
            Gender::Female
        }
    }

    fn get_random_ability(pokemon: &Pokemon) -> String {
        let rng = thread_rng().gen_range(0..100);
        if rng > 95 {
            if let Some(result) = &pokemon.hidden_ability {
                return result.clone();
            }
        }

        if rng > 43 {
            if let Some(result) = &pokemon.ability2 {
                return result.clone();
            }
        }

        pokemon.ability1.clone()
    }

    fn increase_stat(&mut self, stat: &Stat) {
        match stat {
            Stat::Strength => self.strength += 1,
            Stat::Dexterity => self.dexterity += 1,
            Stat::Vitality => self.vitality += 1,
            Stat::Special => self.special += 1,
            Stat::Insight => self.insight += 1,
            _ => panic!("Unexpected combat stat: {}", stat)
        };
    }

    fn increase_social_stat(&mut self, stat: &SocialStat) {
        match stat {
            SocialStat::Tough => self.tough += 1,
            SocialStat::Cool => self.cool += 1,
            SocialStat::Beauty => self.beauty += 1,
            SocialStat::Clever => self.clever += 1,
            SocialStat::Cute => self.cute += 1,
        };
    }

    fn get_rank_from_level(level: u8) -> MysteryDungeonRank {
        match level {
            1 => MysteryDungeonRank::Bronze,
            2..=3 => MysteryDungeonRank::Silver,
            4..=7 => MysteryDungeonRank::Gold,
            8..=15 => MysteryDungeonRank::Platinum,
            _ => MysteryDungeonRank::Diamond,
        }
    }

    fn get_stat(&self, stat: &Stat) -> u8 {
        match stat {
            Stat::Strength => self.strength,
            Stat::Dexterity => self.dexterity,
            Stat::Vitality => self.vitality,
            Stat::Special => self.special,
            Stat::Insight => self.insight,
            Stat::Copy => 0,
            Stat::StrengthOrSpecial => {
                if self.strength > self.special {
                    self.strength
                } else {
                    self.special
                }
            }
        }
    }

    fn get_social_stat(&mut self, stat: &SocialStat) -> u8 {
        match stat {
            SocialStat::Tough => self.tough,
            SocialStat::Cool => self.cool,
            SocialStat::Beauty => self.beauty,
            SocialStat::Clever => self.clever,
            SocialStat::Cute => self.cute,
        }
    }

    pub fn build_string(&self, pokemon: &Pokemon, data: &GameData) -> String{
        let mut result = std::format!("{} ({}) | **Lv.{} ({:?})**\n", self.name, self.gender, self.level, self.rank);
        if let Some(type2) = self.type2 {
            result.push_str(std::format!("**Types**: {} / {}\n", self.type1, type2).as_str());
        } else {
            result.push_str(std::format!("**Type**: {}\n", self.type1).as_str());
        }
        result.push_str(std::format!("```
HP: {}  |  Def: {:.0}  |  SpDef: {:.0}
STR: {:>2} / {:>2}      Tough:  {} / 5
DEX: {:>2} / {:>2}      Cool:   {} / 5
VIT: {:>2} / {:>2}      Beauty: {} / 5
SPE: {:>2} / {:>2}      Clever: {} / 5
INS: {:>2} / {:>2}      Cute:   {} / 5
```",         (self.vitality + pokemon.base_hp) * 2,
                     (self.vitality as f32 * 0.5).ceil(),
                     (self.insight as f32 * 0.5).ceil(),
                self.strength, pokemon.strength.max, self.tough,
                self.dexterity, pokemon.dexterity.max, self.cool,
                self.vitality, pokemon.vitality.max, self.beauty,
                self.special, pokemon.special.max, self.clever,
                self.insight, pokemon.insight.max, self.cute,

        ).as_str());
        result.push_str(std::format!("**Ability**: {}\n*{}*\n", self.ability, data.abilities.get(&self.ability.to_lowercase()).unwrap().effect).as_str());

        result.push_str("## Moves\n");
        for move_name in &self.moves {
            let m = data.moves.get(&move_name.to_lowercase()).unwrap_or_else(|| panic!("Every move should be set! {}", move_name));
            result.push_str(std::format!("**{}** – {} | {} | {}\n", m.name, m.typing, m.category, m.target).as_str());
            if m.damage1.unwrap_or(Stat::Strength) == Stat::Copy {
                result.push_str("ACC: **Copy** | DMG: **Copy** \n");
            }
            else {
                let accuracy = self.calculate_accuracy(m);
                let damage = self.calculate_damage(m);
                if damage > 0 {
                    if m.typing.has_stab(&Some(self.type1)) || m.typing.has_stab(&self.type2) {
                        result.push_str(std::format!("ACC: **{}** | DMG: **{} + STAB**\n", accuracy, damage).as_str());
                    } else {
                        result.push_str(std::format!("ACC: **{}** | DMG: **{}**\n", accuracy, damage).as_str());
                    }
                } else {
                    result.push_str(std::format!("ACC: **{}**\n", accuracy).as_str());
                }
            }
            if let Some(effect) = &m.effect {
                result.push_str(effect.as_str());
                result.push_str("\n\n");
            } else {
                result.push('\n');
            }
        }

        result
    }

    fn calculate_accuracy(&self, m: &Move) -> u8 {
        let mut result = 0;
        if let Some(acc) = m.accuracy1 {
            result += self.get_die_count_for_stat(acc);
        }

        if m.accuracy2.is_some() {
            result += self.rank.die_count();
        }

        result
    }

    fn calculate_damage(&self, m: &Move) -> u8 {
        let mut result = m.power;
        if let Some(stat) = m.damage1 {
            result += self.get_stat(&stat);
        }

        if m.happiness_damage.is_some() {
            result += self.rank.die_count();
        }

        result
    }

    fn get_die_count_for_stat(&self, acc: CombatOrSocialStat) -> u8 {
        match acc {
            CombatOrSocialStat::Strength => self.strength,
            CombatOrSocialStat::Dexterity => self.dexterity,
            CombatOrSocialStat::Vitality => self.vitality,
            CombatOrSocialStat::Special => self.special,
            CombatOrSocialStat::Insight => self.insight,
            CombatOrSocialStat::Tough => self.tough,
            CombatOrSocialStat::Cool => self.cool,
            CombatOrSocialStat::Beauty => self.beauty,
            CombatOrSocialStat::Clever => self.clever,
            CombatOrSocialStat::Cute => self.cute,
            CombatOrSocialStat::Brawl => self.rank.die_count(),
            CombatOrSocialStat::Channel => self.rank.die_count(),
            CombatOrSocialStat::Clash => self.rank.die_count(),
            CombatOrSocialStat::Evasion => self.rank.die_count(),
            CombatOrSocialStat::Alert => self.rank.die_count(),
            CombatOrSocialStat::Athletic => self.rank.die_count(),
            CombatOrSocialStat::Nature => self.rank.die_count(),
            CombatOrSocialStat::Stealth => self.rank.die_count(),
            CombatOrSocialStat::Allure => self.rank.die_count(),
            CombatOrSocialStat::Etiquette => self.rank.die_count(),
            CombatOrSocialStat::Intimidate => self.rank.die_count(),
            CombatOrSocialStat::Perform => self.rank.die_count(),
            CombatOrSocialStat::Will => self.rank.die_count(),
            CombatOrSocialStat::Copied => 0,
            CombatOrSocialStat::ToughOrCute => {
                if self.tough > self.cute {
                    self.tough
                } else {
                    self.cute
                }
            },
            CombatOrSocialStat::MissingBeauty => 5 - self.beauty,
            CombatOrSocialStat::BrawlOrChannel => self.rank.die_count(),
            CombatOrSocialStat::Varies => self.rank.die_count(),
            CombatOrSocialStat::Medicine => self.rank.die_count(),
            CombatOrSocialStat::Empathy => self.rank.die_count(),
            CombatOrSocialStat::Rank => self.rank.die_count(),
        }
    }
}

