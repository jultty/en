use axum::{body::Body, extract::Path, http::Response};
use crate::syntax::content::parsers::line::elements::paragraph::Paragraph;
use crate::syntax::content;

use crate::syntax::content::parsers::word::elements::literal::Literal;
use crate::{formats::populate_graph, handlers, types::Node};

pub async fn node(Path(id): Path<String>) -> Response<Body> {
    let graph = populate_graph();
    let empty_node = Node::new(Some(format!("Could not find node ID {id}.")));
    let node: &Node = graph.nodes.get(&id).unwrap_or(&empty_node);

    let mut context = tera::Context::new();
    context.insert("id", &id);
    context.insert("title", &node.title);
    context.insert("connections", &node.connections.clone());
    context.insert("incoming", &graph.incoming.get(&id));
    context.insert("config", &graph.meta.config);

    let out_text = content::parse::<Paragraph, Literal>(&node.text);
    context.insert("text", &out_text);

    let not_found = *node == empty_node;
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
