#[derive(Debug, serde::Deserialize)]
pub struct CustomItem {
    pub name: String,
    pub price: Option<u16>,
    pub category: String,
    pub description: String,
    pub single_use: bool,
    pub health_restored: Option<u8>,
}
