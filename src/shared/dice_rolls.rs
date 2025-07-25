use crate::Error;
use crate::shared::PoiseContext;
use crate::shared::errors::ParseError;
use crate::shared::utility::button_building;
use poise::CreateReply;
use rand::Rng;
use serenity::all::CreateActionRow;
use std::str::FromStr;

const CRIT: u8 = 6;
const FAIL_THRESHOLD: u8 = 3;

pub const DEFAULT_CRIT_DIE_COUNT: u8 = 3;
pub const DEFAULT_CRIT_DIE_COUNT_OPTION: Option<u8> = Some(DEFAULT_CRIT_DIE_COUNT);

pub fn parse_query(query: &str) -> Result<ParsedRollQuery, Error> {
    let flat_addition: Option<u8>;

    let mut remaining_query = query.to_string();
    if remaining_query.contains('+') {
        let split: Vec<&str> = remaining_query.split('+').collect();
        if remaining_query.starts_with('+') {
            if split.len() > 1 {
                return Err(Box::new(ParseError::new("Unable to parse query.")));
            }

            match u8::from_str(split[0]) {
                Ok(value) => flat_addition = Some(value),
                Err(_) => return Err(Box::new(ParseError::new("Unable to parse query."))),
            }

            remaining_query = String::from("");
        } else {
            if split.len() != 2 {
                return Err(Box::new(ParseError::new("Unable to parse query.")));
            }

            match u8::from_str(split[1]) {
                Ok(value) => flat_addition = Some(value),
                Err(_) => return Err(Box::new(ParseError::new("Unable to parse query."))),
            }
            remaining_query = String::from(split[0]);
        }
    } else {
        flat_addition = None;
    }

    let split: Vec<&str> = remaining_query.split('d').collect();
    if split.len() != 2 {
        let amount = match u8::from_str(&remaining_query) {
            Ok(value) => Some(value),
            Err(_) => return Err(Box::new(ParseError::new("Unable to parse query."))),
        };

        return Ok(ParsedRollQuery::new(
            amount,
            Some(6),
            flat_addition,
            DEFAULT_CRIT_DIE_COUNT_OPTION,
        ));
    }

    let amount = match u8::from_str(split[0]) {
        Ok(value) => Some(value),
        Err(_) => return Err(Box::new(ParseError::new("Unable to parse query."))),
    };

    let sides = match u8::from_str(split[1]) {
        Ok(value) => Some(value),
        Err(_) => return Err(Box::new(ParseError::new("Unable to parse query."))),
    };

    Ok(ParsedRollQuery::new(
        amount,
        sides,
        flat_addition,
        DEFAULT_CRIT_DIE_COUNT_OPTION,
    ))
}

pub async fn execute_query<'a>(ctx: &PoiseContext<'a>, query: &str) -> Result<(), Error> {
    let parsed_query = match parse_query(query) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    execute_roll(ctx, parsed_query).await
}

pub async fn roll<'a>(
    ctx: &PoiseContext<'a>,
    amount: Option<u8>,
    sides: Option<u8>,
    flat_addition: Option<u8>,
) -> Result<(), Error> {
    execute_roll(
        ctx,
        ParsedRollQuery::new(amount, sides, flat_addition, DEFAULT_CRIT_DIE_COUNT_OPTION),
    )
    .await
}

async fn execute_roll<'a>(ctx: &PoiseContext<'a>, query: ParsedRollQuery) -> Result<(), Error> {
    ctx.defer().await?;
    let result = query.execute().message;
    let query_string = query.as_button_callback_query_string();
    ctx.send(
        CreateReply::default()
            .content(result)
            .components(vec![CreateActionRow::Buttons(vec![
                button_building::create_button("Roll again!", query_string.as_str(), false),
            ])]),
    )
    .await?;
    Ok(())
}

#[derive(Default)]
pub struct ParsedRollQuery {
    amount: u8,
    sides: u8,
    flat_addition: u8,

    /// None means this roll cannot crit.
    required_amount_of_6_for_critical_hit: Option<u8>,
}

pub struct RollQueryResult {
    pub success_count: u8,
    pub message: String,
    pub is_critical_hit: bool,
}

impl ParsedRollQuery {
    pub fn new(
        dice: Option<u8>,
        sides: Option<u8>,
        flat_addition: Option<u8>,
        critical_hit_die_count: Option<u8>,
    ) -> Self {
        ParsedRollQuery {
            amount: dice.unwrap_or(1).clamp(0, 100),
            sides: sides.unwrap_or(6).clamp(0, 100),
            flat_addition: flat_addition.unwrap_or(0),
            required_amount_of_6_for_critical_hit: critical_hit_die_count,
        }
    }

    fn as_button_callback_query_string(&self) -> String {
        format!(
            "roll-dice_{}d{}+{}",
            self.amount, self.sides, self.flat_addition
        )
    }

    pub fn execute(&self) -> RollQueryResult {
        let mut results = Vec::new();
        let mut total: u32 = self.flat_addition as u32;
        let mut six_count: u8 = 0;
        let mut successes: u8 = 0;
        {
            // TODO: this is ugly :>
            let mut rng = rand::rng();
            for _ in 0..self.amount {
                let value = rng.random_range(1..self.sides + 1);
                total += value as u32;
                if value > 3 {
                    successes += 1;
                    if value == 6 {
                        six_count += 1;
                    }
                }
                results.push(value);
            }
        }

        let result_list = results
            .iter()
            .map(|x| {
                if self.sides == CRIT {
                    if x == &CRIT {
                        return format!("**__{}__**", x);
                    } else if x > &FAIL_THRESHOLD {
                        return format!("**{}**", x);
                    }
                }

                x.to_string()
            })
            .collect::<Vec<String>>()
            .join(", ");

        let mut message = format!("{}d{}", self.amount, self.sides);

        let mut is_critical_hit = false;
        if self.flat_addition > 0 {
            message.push_str(&format!(
                "+{} — {}+{} = {}",
                self.flat_addition, result_list, self.flat_addition, total
            ));
        } else {
            message.push_str(&format!(" — {}", result_list));
            if self.sides == 6 {
                let success_string: &str;
                if successes == 0 {
                    success_string = "Successes...";
                } else if successes >= 6 {
                    success_string = "Successes!!";
                } else if successes >= 3 {
                    success_string = "Successes!";
                } else if successes == 1 {
                    success_string = "Success.";
                } else {
                    success_string = "Successes.";
                }

                is_critical_hit = if let Some(critical_hit_dies_necessary) =
                    self.required_amount_of_6_for_critical_hit
                {
                    six_count >= critical_hit_dies_necessary
                } else {
                    false
                };

                let crit_string = if is_critical_hit { " **(CRIT)**" } else { "" };

                message.push_str(&format!(
                    "\n**{}** {}{}",
                    successes, success_string, crit_string
                ));
            }
        }

        RollQueryResult {
            success_count: successes,
            message,
            is_critical_hit,
        }
    }
}

pub fn append_crit_stat_if_changed(message: &mut String, crit_6_count: u8) {
    if crit_6_count != DEFAULT_CRIT_DIE_COUNT {
        if crit_6_count > 0 {
            message.push_str(&format!(" | {crit_6_count}x6 required for crit"));
        } else {
            message.push_str(" | guaranteed critical hit");
        }
    }
}
