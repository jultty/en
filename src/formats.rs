use std::collections::HashMap;

use crate::types::*;

pub fn populate_graph() -> Graph {

    let toml_source = match std::fs::read_to_string("./static/graph.toml") {
        Ok(s) => s,
        Err(e) => format!("Error: {e}"),
    };
    let graph = deserialize_graph(Format::Toml, &toml_source);

    let nodes = modulate_nodes(graph.nodes.clone());

    Graph {
        nodes: nodes.clone(),
        incoming: make_incoming(nodes.clone()),
        ..graph
    }
}

fn modulate_nodes(old_nodes: HashMap<String, Node>) -> HashMap<String, Node> {

    let mut nodes: HashMap<String, Node> = HashMap::new();

    for (key, node) in old_nodes.iter() {

        let connections = node.connections.clone().unwrap_or_default();
        let mut vec = connections.clone();

        for (i, edge) in connections.iter().enumerate() {

            let mut new_edge = edge.clone();

            // Populate empty "from" IDs in edges with node's ID
            if edge.from == "" {
                new_edge.from = key.to_string();
            }

            // Flag detached edges
            if ! old_nodes.contains_key(&edge.to) {
                new_edge.detached = true;
            }

            vec[i] = new_edge;

        }

        let new_node = Node {
            connections: Some(vec),
            ..node.clone()
        };
        nodes.insert(key.to_string(), new_node);
    }

    nodes
}

// Construct a HashMap with incoming connections (reversed edges)
fn make_incoming(nodes: HashMap<String, Node>) -> HashMap<String, Vec<Edge>> {

    let mut incoming: HashMap<String, Vec<Edge>> = HashMap::new();
    for node in nodes.clone().into_values() {

        let empty_vec: Vec<Edge> = vec![];
        for edge in node.connections.clone().unwrap_or_default().iter() {

            let vec = incoming.get(&edge.to.clone()).unwrap_or(&empty_vec);
            if vec.contains(edge) {
                vec.clone().extend_from_slice(&[edge.clone()]);
                incoming.insert(edge.to.clone(), vec.clone());
            } else {
                incoming.insert(edge.to.clone(), vec![edge.clone()]);
            }
        }
    }

    incoming
}

pub enum Format {
    Toml,
    Json
}

pub fn serialize_graph(out_format: Format, graph: &Graph) -> String {

    match out_format {
        Format::Toml => {
            match toml::to_string(graph) {
                Ok(s) => s,
                Err(e) => e.to_string(),
            }
        },
        Format::Json => {
            match serde_json::to_string(graph) {
                Ok(s) => s,
                Err(e) => e.to_string(),
            }
        },
    }
}

pub fn deserialize_graph(in_format: Format, serial: &String) -> Graph {

    match in_format {
        Format::Toml => { match toml::from_str(&serial) {
            Ok(g) => g,
            Err(error) => Graph::new(Some(error.to_string()))
        }},
        Format::Json => { match serde_json::from_str(&serial) {
            Ok(g) => g,
            Err(error) => Graph::new(Some(error.to_string()))
        }}
    }
}


