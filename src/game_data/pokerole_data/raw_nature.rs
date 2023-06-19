use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawPokeroleNature {
    pub name: String,
    pub nature: String,
    pub confidence: u8,
    pub keywords: String,
    pub description: String,
}
