use crate::data::Data;
use crate::Error;
use poise::Command;

mod add_quest_participant;
mod create_quest;
mod remove_quest_participant;

pub fn get_all_commands() -> Vec<Command<Data, Error>> {
    vec![
        create_quest::create_quest(),
        add_quest_participant::add_quest_participant(),
        remove_quest_participant::remove_quest_participant(),
    ]
}
