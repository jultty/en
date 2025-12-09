use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Graph {
    pub messages: Vec<String>,
    pub root_node: String,
    pub nodes: HashMap<String, Node>,
    #[serde(skip)]
    pub incoming: HashMap<String, Vec<Edge>>,
}

#[derive(Serialize, Clone, Default, PartialEq, Deserialize)]
pub struct Edge {
    #[serde(default)]
    pub anchor: String,
    #[serde(default)]
    pub from: String,
    pub to: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Node {
    pub title: String,
    pub id: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<Edge>>,
}

impl Graph {
    pub fn new(message: Option<String>) -> Graph {
        Self {
            nodes: HashMap::new(),
            incoming: HashMap::new(),
            root_node: "".to_string(),
            messages: vec![message
                .unwrap_or("This graph is empty or in error".to_string())],
        }
    }

    pub fn get_root(&self) -> Option<Node> {
        match self.nodes.get(&self.root_node) {
            Some(n) => Some(n.clone()),
            None => None,
        }
    }
}

impl Node {
    pub fn new(message: Option<String>) -> Node {
        Self {
            title: "Empty Node".to_string(),
            id: "EmptyNode".to_string(),
            body: match message {
                Some(s) => s,
                None => "Node is empty, missing or wasn't found.".to_string()
            },
            connections: None,
        }
    }
}
