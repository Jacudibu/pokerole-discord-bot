use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawPokeroleItem {
    pub name: String,
    pub pmd_price: Option<u16>,
    pub trainer_price: Option<String>,
    pub health_restored: Option<u8>,
    pub pocket: String,
    pub description: String,
    pub category: String,
    pub one_use: bool,
}
