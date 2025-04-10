use crate::shared::data::Data;
use crate::Error;

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
pub mod helpers;
pub mod metronome;

pub type PoiseContext<'a> = poise::Context<'a, Data, Error>;
