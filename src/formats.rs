use std::collections::HashMap;

use crate::types::*;

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

pub fn populate_graph() -> Graph {
    let toml_source = match std::fs::read_to_string("./static/graph.toml") {
        Ok(s) => s,
        Err(e) => format!("Error: {e}"),
    };
    let graph = deserialize_graph(Format::Toml, &toml_source);
    let mut new_nodes: HashMap<String, Node> = HashMap::new();
    let mut incoming: HashMap<String, Vec<Edge>> = HashMap::new();

    for (key, node) in graph.nodes.iter() {

        let connections = node.connections.clone().unwrap_or_default();
        let mut vec = connections.clone();

        for (i, edge) in connections.iter().enumerate() {

            let mut new_edge = edge.clone();

            if edge.from == "" {
                new_edge.from = key.to_string();
            }

            if ! graph.nodes.contains_key(&edge.to) {
                new_edge.detached = true;
            }

            vec[i] = new_edge;

        }

        let new_node = Node {
            connections: Some(vec),
            ..node.clone()
        };
        new_nodes.insert(key.to_string(), new_node);
    }

    // Construct a HashMap with incoming connections (reversed edges)
    for node in new_nodes.clone().into_values() {

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

    Graph {
        nodes: new_nodes,
        incoming: incoming,
        ..graph
    }

}
