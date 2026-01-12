use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::syntax::content;
pub use {
    node::Node,
    edge::Edge,
    meta::{Meta, Config},
};

pub mod node;
pub mod edge;
pub mod meta;

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Graph {
    pub nodes: HashMap<String, Node>,
    pub root_node: String,
    #[serde(skip_deserializing)]
    pub incoming: HashMap<String, Vec<Edge>>,
    #[serde(skip_deserializing)]
    pub lowercase_keymap: HashMap<String, String>,
    #[serde(default)]
    pub meta: Meta,
}

#[derive(Clone)]
pub struct QueryResult {
    pub node: Option<Node>,
    pub redirect: bool,
}

impl Graph {
    pub fn new(message: Option<&str>) -> Graph {
        Graph {
            nodes: HashMap::default(),
            root_node: "VoidNode".to_string(),
            incoming: HashMap::default(),
            lowercase_keymap: HashMap::default(),
            meta: Meta {
                config: Config::default(),
                version: (0, 1, 0),
                messages: message.map_or(vec![], |m| vec![m.to_string()]),
            },
        }
    }

    pub fn find_node(&self, query: &str) -> QueryResult {
        let collapsed_query = query.trim().replace(" ", "");

        let candidate = if let Some(exact_match) = self.nodes.get(query) {
            QueryResult {
                node: Some(exact_match.clone()),
                redirect: false,
            }
        } else if let Some(lower_key) =
            self.lowercase_keymap.get(&collapsed_query.to_lowercase())
        {
            QueryResult {
                node: self.nodes.get(lower_key).cloned(),
                redirect: true,
            }
        } else {
            QueryResult {
                node: None,
                redirect: false,
            }
        };

        if let Some(candidate_node) = &candidate.node
            && !candidate_node.redirect.is_empty()
        {
            QueryResult {
                node: self.find_node(&candidate_node.redirect).node,
                redirect: true,
            }
        } else {
            candidate
        }
    }

    pub fn get_root(&self) -> Option<Node> {
        self.nodes.get(&self.root_node).cloned()
    }

    pub fn parse(&mut self) {
        self.meta.config.footer_text =
            content::parse(&self.meta.config.footer_text, self);
        self.meta.config.about_text =
            content::parse(&self.meta.config.about_text, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let graph = Graph::new(Some("ISryQFd9peG6eYz9CFRQFWeD1GnPo0oj"));
        assert!(graph.nodes.is_empty());
        assert!(graph.incoming.is_empty());
        assert_eq!(
            graph.meta.messages.first().unwrap(),
            "ISryQFd9peG6eYz9CFRQFWeD1GnPo0oj"
        );
    }
}
