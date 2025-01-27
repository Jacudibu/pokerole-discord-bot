use crate::commands::{Context, Error};
use crate::errors::ParseError;
use crate::helpers;
use poise::CreateReply;
use rand::Rng;
use serenity::all::CreateActionRow;
use std::str::FromStr;

const CRIT: u8 = 6;
const FAIL_THRESHOLD: u8 = 3;

/// Roll dice using a "1d6+4" style text query.
#[poise::command(slash_command)]
pub async fn r(
    ctx: Context<'_>,
    #[description = "1d6+5 will roll 1d6 and add 5."] query: String,
) -> Result<(), Error> {
    execute_query(&ctx, &query).await
}

/// Roll dice by entering die amount, sides and flat addition manually.
#[poise::command(slash_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "How many dies?"]
    #[min = 1_u8]
    #[max = 100_u8]
    dice: Option<u8>,
    #[description = "How many sides?"]
    #[min = 2_u8]
    #[max = 100_u8]
    sides: Option<u8>,
    #[description = "Add a flat value to the result"]
    #[min = 0_u8]
    #[max = 100_u8]
    flat_addition: Option<u8>,
) -> Result<(), Error> {
    execute_parsed(&ctx, ParsedRollQuery::new(dice, sides, flat_addition)).await
}

pub struct ParsedRollQuery {
    amount: u8,
    sides: u8,
    flat_addition: u8,
}

impl ParsedRollQuery {
    pub fn new(dice: Option<u8>, sides: Option<u8>, flat_addition: Option<u8>) -> Self {
        ParsedRollQuery {
            amount: dice.unwrap_or(1).clamp(0, 100),
            sides: sides.unwrap_or(6).clamp(0, 100),
            flat_addition: flat_addition.unwrap_or(0),
        }
    }

    fn as_button_callback_query_string(&self) -> String {
        format!(
            "roll-dice_{}d{}+{}",
            self.amount, self.sides, self.flat_addition
        )
    }

    pub fn execute(&self) -> String {
        let mut results = Vec::new();
        let mut total: u32 = self.flat_addition as u32;
        let mut six_count: u32 = 0;
        let mut successes: u32 = 0;
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

        let mut text = format!("{}d{}", self.amount, self.sides);

        if self.flat_addition > 0 {
            text.push_str(&format!(
                "+{} — {}+{} = {}",
                self.flat_addition, result_list, self.flat_addition, total
            ));
        } else {
            text.push_str(&format!(" — {}", result_list));
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

                let crit_string = if six_count >= 3 { " **(CRIT)**" } else { "" };

                text.push_str(&format!(
                    "\n**{}** {}{}",
                    successes, success_string, crit_string
                ));
            }
        }

        text
    }
}

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

        return Ok(ParsedRollQuery::new(amount, Some(6), flat_addition));
    }

    let amount = match u8::from_str(split[0]) {
        Ok(value) => Some(value),
        Err(_) => return Err(Box::new(ParseError::new("Unable to parse query."))),
    };

    let sides = match u8::from_str(split[1]) {
        Ok(value) => Some(value),
        Err(_) => return Err(Box::new(ParseError::new("Unable to parse query."))),
    };

    Ok(ParsedRollQuery::new(amount, sides, flat_addition))
}

pub async fn execute_query<'a>(ctx: &Context<'a>, query: &str) -> Result<(), Error> {
    let parsed_query = match parse_query(query) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    execute_parsed(ctx, parsed_query).await
}

async fn execute_parsed<'a>(ctx: &Context<'a>, query: ParsedRollQuery) -> Result<(), Error> {
    ctx.defer().await?;
    let result = query.execute();
    let query_string = query.as_button_callback_query_string();
    ctx.send(
        CreateReply::default()
            .content(result)
            .components(vec![CreateActionRow::Buttons(vec![
                helpers::create_button("Roll again!", query_string.as_str(), false),
            ])]),
    )
    .await?;
    Ok(())
}
