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

    #[serde(default)]
    pub connections: HashMap<String, Edge>,

    #[serde(default)]
    pub stats: Stats,
}

#[derive(Serialize, Deserialize, Clone, Default, Eq, PartialEq, Debug)]
pub struct Stats {
    pub outgoing: u32,
    pub incoming: u32,
}

impl Node {
    pub fn not_found(message: Option<String>) -> Node {
        Node {
            id: "404".to_string(),
            title: "Not Found".to_string(),
            text: match message {
                Some(s) => s,
                None => "Node not found.".to_string(),
            },
            connections: HashMap::default(),
            links: vec![],
            redirect: String::default(),
            hidden: false,
            summary: String::default(),
            stats: Stats::default(),
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut meta_elements: Vec<String> = vec![];

        if !self.title.is_empty() {
            meta_elements.push(format!("title:'{}'", self.title));
        }

        if !self.text.is_empty() {
            meta_elements.push(format!("text:{}l", self.text.len()));
        }

        if !self.summary.is_empty() {
            meta_elements.push(format!("summary:{}", self.summary.len()));
        }

        if !self.redirect.is_empty() {
            meta_elements.push(format!("redirect:{}", self.redirect));
        }

        let links = self.links.len();
        if links > 0 {
            meta_elements.push(format!("links:{links}"));
        }

        if self.hidden {
            meta_elements.push(String::from("hidden"));
        }

        let meta = meta_elements.join(" ");

        write!(f, "Node {} [{meta}]", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_node_message() {
        let node = Node::not_found(None);
        assert_eq!(node.text, "Node not found.");
    }

    #[test]
    fn display() {
        let mut node = Node::not_found(None);
        assert_eq!(format!("{node}"), "Node 404 [title:'Not Found' text:15l]");

        let summary = "X2hSwanDoLdqLZNnYJagcWKFJVAx5TGF";
        node.summary = String::from(summary);
        assert_eq!(
            format!("{node}"),
            format!(
                "Node 404 [title:'Not Found' text:15l summary:{}]",
                summary.len()
            )
        );

        let redirect = "ukfF3kz130oUzT2ushBIvEHx8xoY8ke0";
        node.redirect = String::from(redirect);
        assert_eq!(
            format!("{node}"),
            format!(
                "Node 404 [title:'Not Found' text:15l summary:{} redirect:{redirect}]",
                summary.len(),
            )
        );

        node.links.push(String::from("1"));
        node.links.push(String::from("2"));
        node.links.push(String::from("3"));

        assert_eq!(
            format!("{node}"),
            format!(
                "Node 404 [\
                    title:'Not Found' \
                    text:15l summary:{} \
                    redirect:{redirect} \
                    links:{}\
                    ]",
                summary.len(),
                node.links.len(),
            )
        );

        node.hidden = true;

        assert_eq!(
            format!("{node}"),
            format!(
                "Node 404 [\
                    title:'Not Found' \
                    text:15l summary:{} \
                    redirect:{redirect} \
                    links:{} \
                    hidden\
                    ]",
                summary.len(),
                node.links.len(),
            )
        );
    }
}
