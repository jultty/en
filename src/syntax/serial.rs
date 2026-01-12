use std::collections::HashMap;

use crate::{
    syntax::{
        command::Arguments,
        content::{
            self,
            parser::{flatten, Token, token::Anchor},
        },
    },
    graph::{Edge, Graph, Node},
};

pub fn populate_graph() -> Graph {
    let args = Arguments::default().parse();
    let toml_source = match std::fs::read_to_string(args.graph_path) {
        Ok(s) => s,
        Err(e) => format!("Error: {e}"),
    };

    let graph = deserialize_graph(&Format::TOML, &toml_source);
    modulate_graph(&graph)
}

fn modulate_graph(in_graph: &Graph) -> Graph {
    let nodes = modulate_nodes(in_graph);

    let mut graph = Graph {
        incoming: make_incoming(&nodes),
        lowercase_keymap: map_lowercase_keys(&nodes),
        nodes,
        ..in_graph.to_owned()
    };

    graph.parse();
    graph
}

fn modulate_nodes(graph: &Graph) -> HashMap<String, Node> {
    let old_nodes = graph.nodes.clone();
    let mut nodes: HashMap<String, Node> = HashMap::default();

    for (key, node) in old_nodes.clone() {
        let connections = node.connections.clone().unwrap_or_default();
        let mut new_edges = connections.clone();

        // Parse node text
        let (text, tokens) = content::rich_parse(&node.text, graph);

        // Modulate connections
        for (i, edge) in connections.iter().enumerate() {
            let mut new_edge = edge.clone();

            // Populate empty "from" IDs in edges with node's ID
            if edge.from.is_empty() {
                new_edge.from.clone_from(&key);
            }

            // Flag detached edges
            if !old_nodes.contains_key(&edge.to) {
                new_edge.detached = true;
            }

            if let Some(e) = new_edges.get_mut(i) {
                *e = new_edge;
            }
        }

        // Create connections for each link
        for link in &node.links {
            new_edges.push(Edge {
                from: key.clone(),
                to: link.clone(),
                anchor: String::default(),
                detached: !old_nodes.clone().contains_key(link),
            });
        }

        // Create connections for each anchor
        let parsed_anchors =
            tokens.iter().filter(|t| matches!(t, Token::Anchor(_)));

        let mut anchors: Vec<Anchor> = vec![];
        for anchor in parsed_anchors {
            if let Token::Anchor(a) = anchor {
                anchors.push(*a.clone());
            }
        }

        for anchor in anchors {
            if let Some(anchor_node) = anchor.node() {
                new_edges.push(Edge {
                    from: key.clone(),
                    to: anchor_node.id,
                    anchor: anchor.text(),
                    detached: false,
                });
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

        let new_node = Node {
            id: key.clone(),
            title: new_title,
            summary: flatten(&summary, graph),
            connections: Some(new_edges),
            text,
            ..node.clone()
        };

        nodes.insert(key.clone(), new_node);
    }

    nodes
}

pub enum Format {
    TOML,
    JSON,
}

pub fn serialize_graph(out_format: &Format, graph: &Graph) -> String {
    match *out_format {
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

pub fn deserialize_graph(in_format: &Format, serial: &str) -> Graph {
    match *in_format {
        Format::TOML => match toml::from_str(serial) {
            Ok(g) => g,
            Err(error) => Graph::new(Some(&error.to_string())),
        },
        Format::JSON => match serde_json::from_str(serial) {
            Ok(g) => g,
            Err(error) => Graph::new(Some(&error.to_string())),
        },
    }
}

// Construct a HashMap with incoming connections (reversed edges)
fn make_incoming(nodes: &HashMap<String, Node>) -> HashMap<String, Vec<Edge>> {
    let mut incoming: HashMap<String, Vec<Edge>> = HashMap::default();

    for node in nodes.clone().into_values() {
        let empty_vec: Vec<Edge> = vec![];
        for edge in &node.connections.clone().unwrap_or_default() {
            let mut edges =
                incoming.get(&edge.to.clone()).unwrap_or(&empty_vec).clone();
            edges.extend_from_slice(std::slice::from_ref(edge));
            incoming.insert(edge.to.clone(), edges.clone());
        }
    }

    incoming
}

fn map_lowercase_keys(
    source_map: &HashMap<String, Node>,
) -> HashMap<String, String> {
    let mut out_map: HashMap<String, String> = HashMap::default();
    let keys = source_map.keys();
    for key in keys {
        out_map.insert(key.clone().to_lowercase(), key.clone());
    }
    out_map
}

#[cfg(test)]
mod tests {
    use super::*;

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
                    "connections": []
                }
            },
            "root_node": "JSON"
        }
        "#;

        let graph = deserialize_graph(&Format::JSON, json);
        assert!(graph.meta.messages.is_empty());
    }

    #[test]
    fn bad_json() {
        let graph = deserialize_graph(&Format::JSON, ":::");
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

        let graph = populate_graph();
        let message = graph.meta.messages.first().unwrap();
        assert!(message.contains("TOML parse error"));
        assert!(message.contains("No such file or directory"));

        assert!(std::env::set_current_dir(original_working_directory).is_ok());
    }
}
