use crate::Error;
use crate::shared::data::Data;

pub mod action_log;
pub mod cache;
pub mod character;
pub mod character_stats;
pub mod clunky_stuff;
pub mod constants;
pub mod data;
pub mod dice_rolls;
pub mod discord_error_codes;
pub mod emoji;
pub mod enums;
pub mod errors;
pub mod game_data;
pub mod metronome;
pub mod retire_character;
pub mod utility;

pub type PoiseContext<'a> = poise::Context<'a, Data, Error>;

/// Just trying to make it easier to differentiate between Poise and Serenity context.
pub type SerenityContext = serenity::all::Context;
