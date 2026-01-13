use std::{collections::HashMap, path::PathBuf};

use serde::{Serialize, Deserialize};

use crate::syntax::{
    command::Arguments,
    content::{
        self,
        parser::{flatten, Token, token::Anchor},
    },
};
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
    #[serde(default)]
    pub stats: Stats,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Stats {
    pub detached: HashMap<String, u32>,
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
    pub fn error(message: Option<&str>) -> Graph {
        let graph = Graph::default();
        Graph {
            meta: Meta {
                messages: message.map_or(vec![], |m| vec![m.to_string()]),
                ..graph.meta
            },
            ..graph
        }
    }

    /// Loads a TOML file from the default location and returns a modulated Graph
    pub fn load() -> Graph {
        Self::load_file("")
    }

    /// Takes a file path to a TOML file and returns a modulated Graph
    ///
    /// If `path` is an empty string, it will fallback to CLI arguments
    pub fn load_file(path: &str) -> Graph {
        let mut graph = if path.is_empty() {
            Self::read_file(None)
        } else {
            Self::read_file(Some(path))
        };
        graph.modulate();
        graph
    }

    ///  Reads a TOML fie into a Graph without modulating it
    pub fn read_file(in_path: Option<&str>) -> Graph {
        let cli_path = Arguments::default().parse().graph_path;
        let path = in_path.map_or(cli_path, PathBuf::from);

        let toml_source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => format!("Error: {e}"),
        };

        Self::from_serial(&toml_source, &Format::TOML)
    }

    pub fn from_serial(serial: &str, format: &Format) -> Graph {
        match *format {
            Format::TOML => match toml::from_str(serial) {
                Ok(g) => g,
                Err(error) => Graph::error(Some(&error.to_string())),
            },
            Format::JSON => match serde_json::from_str(serial) {
                Ok(g) => g,
                Err(error) => Graph::error(Some(&error.to_string())),
            },
        }
    }

    pub fn to_serial(graph: &Graph, format: &Format) -> String {
        match *format {
            Format::TOML => match toml::to_string(graph) {
                Ok(s) => s,
                Err(e) => e.to_string(),
            },
            Format::JSON => match serde_json::to_string(graph) {
                Ok(s) => s,
                Err(e) => e.to_string(),
            },
        }
    }

    pub fn modulate(&mut self) {
        self.map_lowercase_keys();
        self.modulate_nodes();
        self.modulate_edges();
        self.map_incoming();
        self.parse_config();
    }

    // Construct a HashMap with incoming connections (reversed edges)
    fn map_incoming(&mut self) {
        for node in self.nodes.clone().into_values() {
            for edge in node.connections.clone().unwrap_or_default().values() {
                let mut edges = self
                    .incoming
                    .get(&edge.to.clone())
                    .unwrap_or(&vec![])
                    .clone();
                edges.extend_from_slice(std::slice::from_ref(edge));
                self.incoming.insert(edge.to.clone(), edges.clone());
            }
        }
    }

    fn modulate_nodes(&mut self) {
        let in_nodes = self.nodes.clone();

        for (key, node) in in_nodes.clone() {
            let connections = node.connections.clone().unwrap_or_default();
            let mut new_edges = connections.clone();

            // Modulate connections
            for (connection_key, edge) in connections {
                let mut new_edge = edge.clone();

                // Populate empty "from" IDs in edges with node's ID
                if edge.from.is_empty() {
                    new_edge.from.clone_from(&connection_key);
                }

                // Flag detached edges
                if in_nodes.contains_key(&edge.to) {
                    new_edge.detached = false;
                } else {
                    new_edge.detached = true;
                    self.increment_detached(&key);
                }

                if let Some(e) = new_edges.get_mut(&connection_key) {
                    *e = new_edge;
                }
            }

            // Create connections for each link
            for link in &node.links {
                let detached = !in_nodes.contains_key(link);
                new_edges.insert(
                    link.clone(),
                    Edge {
                        from: key.clone(),
                        to: link.clone(),
                        detached,
                    },
                );

                if detached {
                    self.increment_detached(&key);
                }
            }

            // Populate empty titles with IDs
            let new_title = if node.title.is_empty() {
                key.clone()
            } else {
                node.title.clone()
            };

            // Populate empty summaries with the leading part of the node text
            let summary = if node.summary.is_empty() {
                let first_line = if let Some(first) =
                    node.text.lines().find(|s| !s.is_empty())
                {
                    String::from(first)
                } else {
                    node.text.clone()
                };

                let mut candidate =
                    if let Some(dot_split) = first_line.split_once('.') {
                        format!("{}.", dot_split.0)
                    } else {
                        first_line
                    };

                if candidate.len() > 300 {
                    candidate.truncate(300);
                    candidate.push('…');
                }
                candidate
            } else {
                node.summary.clone()
            };

            // Assemble new node
            let new_node = Node {
                id: key.clone(),
                title: new_title,
                summary: flatten(&summary, self),
                connections: Some(new_edges),
                ..node.clone()
            };

            self.nodes.insert(key.clone(), new_node);
        }
    }

    fn modulate_edges(&mut self) {
        let graph = self.clone();
        let iterator = self.nodes.iter_mut();
        for (key, node) in iterator {
            // Parse node text
            let parse_output = content::rich_parse(&node.text, &graph);
            node.text = parse_output.text.unwrap_or_default();

            // Create connections for each anchor
            let mut parsed_anchor_tokens = parse_output
                .tokens
                .iter()
                .filter(|t| matches!(t, Token::Anchor(_)))
                .cloned()
                .collect::<Vec<Token>>();
            parsed_anchor_tokens.extend_from_slice(
                &parse_output
                    .format_tokens
                    .iter()
                    .filter(|t| matches!(t, Token::Anchor(_)))
                    .cloned()
                    .collect::<Vec<Token>>(),
            );
            let mut anchors: Vec<Anchor> = vec![];
            for token in parsed_anchor_tokens {
                if let Token::Anchor(token_data) = token {
                    anchors.push(*token_data.clone());
                }
            }

            for anchor in anchors {
                if let Some(anchor_node) = anchor.node() {
                    if let Some(ref mut connections) = node.connections {
                        connections.insert(
                            anchor_node.id.clone(),
                            Edge {
                                from: key.clone(),
                                to: anchor_node.id,
                                detached: false,
                            },
                        );
                    }
                } else {
                    if let Some(destination) = anchor.destination()
                        && !anchor.external()
                    {
                        self.stats
                            .detached
                            .entry(
                                destination
                                    .trim_start_matches("/node/")
                                    .to_string(),
                            )
                            .and_modify(|count| {
                                *count = count.saturating_add(1)
                            })
                            .or_insert(1);
                    }
                }
            }
        }
    }

    fn increment_detached(&mut self, node_id: &str) {
        self.stats
            .detached
            .entry(node_id.to_string())
            .and_modify(|count| *count = count.saturating_add(1))
            .or_insert(1);
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

    pub fn parse_config(&mut self) {
        self.meta.config.footer_text =
            content::parse(&self.meta.config.footer_text, self);
        self.meta.config.about_text =
            content::parse(&self.meta.config.about_text, self);
    }
}

pub enum Format {
    TOML,
    JSON,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let graph = Graph::error(Some("ISryQFd9peG6eYz9CFRQFWeD1GnPo0oj"));
        assert!(graph.nodes.is_empty());
        assert!(graph.incoming.is_empty());
        assert_eq!(
            graph.meta.messages.first().unwrap(),
            "ISryQFd9peG6eYz9CFRQFWeD1GnPo0oj"
        );
    }

    #[test]
    fn good_json() {
        let json = r#"
        {
            "nodes": {
                "JSON": {
                    "text": "",
                    "title": "JSON",
                    "links": [],
                    "id": "JSON",
                    "hidden": false,
                    "connections": {}
                }
            },
            "root_node": "JSON"
        }
        "#;

        let graph = Graph::from_serial(json, &Format::JSON);
        assert!(graph.meta.messages.is_empty());
    }

    #[test]
    fn bad_json() {
        let graph = Graph::from_serial(":::", &Format::JSON);
        let message = graph.meta.messages.first().unwrap();
        assert!(message.contains("expected value at line 1 column 1"));
    }
}

#[cfg(test)]
mod serial_tests {
    use super::*;

    #[test]
    fn bad_graph_path() {
        let original_working_directory = std::env::current_dir().unwrap();

        assert!(
            std::env::set_current_dir(std::path::Path::new(
                "tests/mocks/no_graph"
            ))
            .is_ok()
        );

        let graph = Graph::load();
        let message = graph.meta.messages.first().unwrap();
        assert!(message.contains("TOML parse error"));
        assert!(message.contains("No such file or directory"));

        assert!(std::env::set_current_dir(original_working_directory).is_ok());
    }
}
