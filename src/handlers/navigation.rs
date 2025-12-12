use axum::{
    body::Body,
    http::{Response},
    response::Redirect,
    Form,
};

use crate::{formats::populate_graph, types::Node, handlers};

pub async fn nexus(template: &str) -> Response<Body> {
    let mut context = tera::Context::new();
    let graph = populate_graph();
    let root_node = graph.get_root().unwrap_or_default();
    let nodes: Vec<Node> = graph.nodes.into_values().collect();

    context.insert("nodes", &nodes);
    context.insert("root_node", &root_node);

    handlers::template::by_filename(template, &context, 500, None, false)
}

pub async fn search(Form(query): Form<Query>) -> Redirect {
    Redirect::permanent(format!("/node/{}", query.node).as_str())
}

#[derive(serde::Deserialize)]
pub struct Query {
    node: String,
}
