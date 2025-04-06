use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CustomAbility {
    pub name: String,
    pub description: String,
    pub effect: String,
}
