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

    // If an edge has no "from" ID, default to its node's ID
    for (key, node) in graph.nodes.iter() {

        let mut new_node = node.clone();
        let connections = node.connections.clone().unwrap_or_default();

        for (i, edge) in connections.iter().enumerate() {
            if edge.from == "" {
                let new_edge = Edge {
                    from: key.to_string(),
                    ..edge.clone()
                };
                let mut vec = connections.clone();
                vec[i] = new_edge;
                new_node = Node {
                    connections: Some(vec),
                    ..node.clone()
                };
            }
        }

        new_nodes.insert(key.to_string(), new_node.clone());
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
