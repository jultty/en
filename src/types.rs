use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Graph {
    pub nodes: HashMap<String, Node>,
    pub root_node: String,
    #[serde(default)]
    pub messages: Vec<String>,
    #[serde(skip_deserializing)]
    pub incoming: HashMap<String, Vec<Edge>>,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Node {
    pub text: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub links: Vec<String>,
    #[serde(default)]
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<Edge>>,
}

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

impl Graph {
    pub fn new(message: Option<String>) -> Graph {
        Self {
            nodes: HashMap::new(),
            root_node: "VoidNode".to_string(),
            incoming: HashMap::new(),
            messages: vec![
                message
                    .unwrap_or("This graph is empty or in error".to_string()),
            ],
        }
    }

    pub fn get_root(&self) -> Option<Node> {
        self.nodes.get(&self.root_node).cloned()
    }
}

impl Node {
    pub fn new(message: Option<String>) -> Node {
        Self {
            id: "VoidNode".to_string(),
            title: "Pure Void".to_string(),
            text: match message {
                Some(s) => s,
                None => "Node is empty, missing or wasn't found.".to_string(),
            },
            connections: None,
            links: vec![],
        }
    }
}
