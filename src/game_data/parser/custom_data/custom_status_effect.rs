#[derive(Debug, serde::Deserialize)]
pub struct CustomStatusEffect {
    pub name: String,
    pub description: String,
    pub resist: String,
    pub effect: String,
    pub duration: String,
}
