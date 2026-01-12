use std::collections::HashMap;

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
    pub connections: Option<HashMap<String, Edge>>,
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

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut meta = String::default();
        if self.title.is_empty() {
            meta.push_str("title:none");
        } else {
            meta.push_str(&format!("title:'{}'", self.title));
        }
        if self.text.is_empty() {
            meta.push_str(" text:none");
        } else {
            meta.push_str(&format!(" text:{}l", self.text.len()));
        }
        if self.summary.is_empty() {
            meta.push_str(" summary:none");
        } else {
            meta.push_str(&format!(" summary:{}", self.summary.len()));
        }
        if self.redirect.is_empty() {
            meta.push_str(" redirect:none");
        } else {
            meta.push_str(&format!(" redirect:{}", self.redirect));
        }
        let links = self.links.len();
        if links > 0 {
            meta.push_str(&format!(" links:{links}"));
        }
        if self.hidden {
            meta.push_str(" hidden");
        }
        write!(f, "Node: ID '{}' {meta}", self.id)
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
