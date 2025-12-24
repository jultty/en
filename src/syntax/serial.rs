use std::collections::HashMap;

use crate::{
    syntax::command::Arguments,
    types::{Edge, Graph, Node},
};

pub fn populate_graph() -> Graph {
    let args = Arguments::new().parse();
    let toml_source = match std::fs::read_to_string(args.graph_path) {
        Ok(s) => s,
        Err(e) => format!("Error: {e}"),
    };
    let graph = deserialize_graph(&Format::Toml, &toml_source);

    let nodes = modulate_nodes(&graph.nodes);

    Graph {
        nodes: nodes.clone(),
        incoming: make_incoming(&nodes),
        lowercase_keymap: map_lowercase_keys(&nodes),
        ..graph
    }
}

fn map_lowercase_keys(
    source_map: &HashMap<String, Node>,
) -> HashMap<String, String> {
    let mut out_map: HashMap<String, String> = HashMap::new();
    let keys = source_map.keys();
    for key in keys {
        out_map.insert(key.clone().to_lowercase(), key.clone());
    }
    out_map
}

fn modulate_nodes(old_nodes: &HashMap<String, Node>) -> HashMap<String, Node> {
    let mut nodes: HashMap<String, Node> = HashMap::new();

    for (key, node) in old_nodes {
        let connections = node.connections.clone().unwrap_or_default();
        let mut new_edges = connections.clone();

        for (i, edge) in connections.iter().enumerate() {
            let mut new_edge = edge.clone();

            // Populate empty "from" IDs in edges with node's ID
            if edge.from.is_empty() {
                new_edge.from.clone_from(key);
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
                anchor: String::new(),
                detached: !old_nodes.contains_key(link),
            });
        }

        // Populate empty titles with IDs
        let new_title = if node.title.is_empty() {
            key.clone()
        } else {
            node.title.clone()
        };

        let new_node = Node {
            id: key.clone(),
            title: new_title,
            connections: Some(new_edges),
            ..node.clone()
        };

        nodes.insert(key.clone(), new_node);
    }

    nodes
}

// Construct a HashMap with incoming connections (reversed edges)
fn make_incoming(nodes: &HashMap<String, Node>) -> HashMap<String, Vec<Edge>> {
    let mut incoming: HashMap<String, Vec<Edge>> = HashMap::new();

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

pub enum Format {
    Toml,
    Json,
}

pub fn serialize_graph(out_format: &Format, graph: &Graph) -> String {
    match *out_format {
        Format::Toml => match toml::to_string(graph) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        },
        Format::Json => match serde_json::to_string(graph) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        },
    }
}

pub fn deserialize_graph(in_format: &Format, serial: &str) -> Graph {
    match *in_format {
        Format::Toml => match toml::from_str(serial) {
            Ok(g) => g,
            Err(error) => Graph::new(Some(error.to_string())),
        },
        Format::Json => match serde_json::from_str(serial) {
            Ok(g) => g,
            Err(error) => Graph::new(Some(error.to_string())),
        },
    }
}
