use crate::shared::enums::{MoveCategory, MoveType};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawPokeroleMove {
    pub name: String,
    pub r#type: MoveType,
    pub power: u8,
    pub damage1: String,
    pub damage2: String,
    pub accuracy1: String,
    pub accuracy2: String,
    pub target: String,
    pub effect: String,
    pub description: String,
    //pub attributes - includes stuff like never_fail: bool. But that's already written in effect.
    //pub added_effects - includes stuff like stat changes. PARSEABLE stat changes! But they are already written in effect.
    pub category: MoveCategory,
}
