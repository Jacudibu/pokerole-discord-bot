use crate::shared::data::Data;
use crate::Error;
use poise::Command;

mod add_quest_participant;
mod complete_quest;
mod create_quest;
mod remove_quest_participant;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![
        add_quest_participant::add_quest_participant(),
        complete_quest::complete_quest(),
        create_quest::create_quest(),
        remove_quest_participant::remove_quest_participant(),
    ]
}
