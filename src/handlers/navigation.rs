use axum::{
    body::Body,
    http::{Response},
    response::Redirect,
    Form,
};

use crate::{
    formats::populate_graph,
    handlers,
    syntax::content::{
        self,
        parsers::{
            line::elements::{paragraph::Paragraph, span::Span},
            compound::elements::literal::Literal,
        },
    },
    types::{Config, Node},
};

#[expect(clippy::unused_async)]
pub async fn page(template: &str) -> Response<Body> {
    let mut context = tera::Context::new();
    let graph = populate_graph();
    let root_node = graph.get_root().unwrap_or_default();
    let nodes: Vec<Node> = graph.nodes.into_values().collect();

    context.insert("nodes", &nodes);
    context.insert("root_node", &root_node);
    context.insert("config", &graph.meta.config.parse_text());

    handlers::template::by_filename(template, &context, 500, None, false)
}

pub async fn search(Form(query): Form<Query>) -> Redirect {
    Redirect::permanent(format!("/node/{}", query.node).as_str())
}

#[derive(serde::Deserialize)]
pub struct Query {
    node: String,
}
