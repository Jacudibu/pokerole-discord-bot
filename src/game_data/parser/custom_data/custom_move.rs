use crate::enums::{MoveCategory, MoveType};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CustomMove {
    pub name: String,
    pub r#type: MoveType,
    pub power: Option<u8>,
    pub damage: Option<String>,
    pub accuracy: String,
    pub target: String,
    pub effect: String,
    pub description: String,
    pub category: MoveCategory,
}
