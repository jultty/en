use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::syntax::content;
use crate::prelude::*;
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

#[derive(Clone, Default, Debug)]
pub struct QueryResult {
    pub node: Option<Node>,
    pub redirect: bool,
    pub exact: bool,
}

impl std::fmt::Display for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let meta = if self.redirect { "[redirect] " } else { "" };
        let node = if let Some(n) = &self.node {
            n.id.clone()
        } else {
            String::from("No Match")
        };
        write!(f, "QueryResult: {meta}{node}")
    }
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

    pub fn map_lowercase_keys(&mut self) {
        for key in self.nodes.keys() {
            self.lowercase_keymap
                .insert(key.clone().to_lowercase(), key.clone());
        }
    }

    pub fn find_node(&self, query: &str) -> QueryResult {
        let collapsed_query = query.trim().replace(" ", "");

        if query == collapsed_query {
            log!("Chasing candidate for query {query}");
        } else {
            log!(
                "Chasing candidate for query {query}, collapsed {collapsed_query}"
            );
        }

        let candidate = if let Some(exact_match) = self.nodes.get(query) {
            log!("Elected exact match {exact_match}");
            QueryResult {
                node: Some(exact_match.clone()),
                exact: true,
                redirect: false,
            }
        } else if let Some(lower_key) =
            self.lowercase_keymap.get(&collapsed_query.to_lowercase())
        {
            log!("Elected non-exact match through lower key {lower_key}");
            QueryResult {
                node: self.nodes.get(lower_key).cloned(),
                exact: false,
                redirect: false,
            }
        } else {
            log!("No candidate found");
            QueryResult::default()
        };

        if let Some(candidate_node) = &candidate.node
            && !candidate_node.redirect.is_empty()
        {
            log!("Recursing: candidate is a redirect");
            if let Some(recursive_match) =
                self.find_node(&candidate_node.redirect).node
            {
                QueryResult {
                    node: Some(recursive_match),
                    exact: false,
                    redirect: true,
                }
            } else {
                QueryResult::default()
            }
        } else {
            log!("Returning candidate {candidate}");
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
