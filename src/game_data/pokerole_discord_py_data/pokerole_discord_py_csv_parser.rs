use serde::Deserialize;
use crate::csv_utils;

#[derive(Debug, Deserialize)]
pub struct RawPokeWeather {
    pub name: String,
    pub description: String,
    pub effect: String,
}

#[derive(Debug, Deserialize)]
pub struct RawPokeStatus {
    pub name: String,
    pub description: String,
    pub resist: String,
    pub effect: String,
    pub duration: String,
}

pub struct RawPokeroleDiscordPyCsvData {
    pub weather: Vec<RawPokeWeather>,
    pub status_effects: Vec<RawPokeStatus>,
}

pub fn parse(path_to_repo: &str) -> RawPokeroleDiscordPyCsvData {
    RawPokeroleDiscordPyCsvData {
        weather: csv_utils::load_csv_with_custom_headers(path_to_repo.to_owned() + "weather.csv", vec![
            "name",
            "description",
            "effect"
        ]),
        status_effects: csv_utils::load_csv_with_custom_headers(path_to_repo.to_owned() + "status.csv", vec![
            "name",
            "description",
            "resist",
            "effect",
            "duration",
        ]),
    }
}
