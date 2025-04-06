#[derive(Debug, serde::Deserialize)]
pub struct CustomWeather {
    pub name: String,
    pub description: String,
    pub effect: String,
}
