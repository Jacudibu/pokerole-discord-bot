#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
pub enum PokeRoleRank {
    Starter,
    Beginner,
    Amateur,
    Ace,
    Pro,
    Master,
    Champion,
}
