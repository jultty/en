use serde::{Serialize, Deserialize};

use super::edge::Edge;

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Node {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub links: Vec<String>,
    #[serde(default)]
    pub redirect: String,
    #[serde(default)]
    pub hidden: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<Edge>>,
}

impl Node {
    pub fn new(message: Option<String>) -> Node {
        Node {
            id: "404".to_string(),
            title: "Not Found".to_string(),
            text: match message {
                Some(s) => s,
                None => "Node not found.".to_string(),
            },
            connections: None,
            links: vec![],
            redirect: String::default(),
            hidden: false,
            summary: String::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_node_message() {
        let node = Node::new(None);
        assert_eq!(node.text, "Node not found.");
    }
}
