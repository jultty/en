use axum::{body::Body, extract::Path, http::Response};

use crate::{formats::populate_graph, types::Node, handlers};

pub async fn node(Path(id): Path<String>) -> Response<Body> {
    let mut context = tera::Context::new();

    let graph = populate_graph();
    let nodes = graph.nodes;
    let empty_node =
        Node::new(Some(format!("Could not find node with ID {id}.")));

    let node: &Node = nodes.get(&id).unwrap_or(&empty_node);

    context.insert("id", &id);
    context.insert("title", &node.title);
    context.insert("text", &node.text);
    context.insert("connections", &node.connections.clone());
    context.insert("incoming", &graph.incoming.get(&id));

    let not_found = node.clone() == empty_node;
    let template_name = "node.html".to_string();

    handlers::template::by_filename(
        &template_name,
        &context,
        if not_found { 404 } else { 500 },
        Some(
            format!(
                "Failed to generate page for node {} (ID {}).\n\
                    Node struct: <pre>{:#?}</pre>",
                node.title, id, node
            )
            .to_owned(),
        ),
        not_found,
    )
}
