#[derive(Debug, serde::Deserialize)]
pub struct CustomPotion {
    pub name: String,
    pub description: String,
    pub effect: String,
    pub recipes: Vec<String>,
}
