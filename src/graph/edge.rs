use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Edge {
    pub to: String,
    #[serde(default)]
    pub anchor: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub detached: bool,
}
