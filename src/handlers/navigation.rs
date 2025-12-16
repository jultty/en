use axum::{
    body::Body,
    http::{Response},
    response::Redirect,
    Form,
};

use crate::{
    formats::populate_graph,
    handlers,
    syntax::content::parser,
    types::{Config, Node},
    syntax::content::elements::{span::Span, paragraph::Paragraph},
};

#[expect(clippy::unused_async)]
pub async fn page(template: &str) -> Response<Body> {
    let mut context = tera::Context::new();
    let graph = populate_graph();
    let root_node = graph.get_root().unwrap_or_default();
    let nodes: Vec<Node> = graph.nodes.into_values().collect();

    context.insert("nodes", &nodes);
    context.insert("root_node", &root_node);

    let text_parsed_config = Config {
        footer_text: parser::read::<Span>(&graph.meta.config.footer_text),
        about_text: parser::read::<Paragraph>(&graph.meta.config.about_text),
        ..graph.meta.config
    };

    context.insert("config", &text_parsed_config);

    handlers::template::by_filename(template, &context, 500, None, false)
}

pub async fn search(Form(query): Form<Query>) -> Redirect {
    Redirect::permanent(format!("/node/{}", query.node).as_str())
}

#[derive(serde::Deserialize)]
pub struct Query {
    node: String,
}
