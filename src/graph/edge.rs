use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Edge {
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub detached: bool,
}
