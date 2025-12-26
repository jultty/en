use axum::{
    body::Body,
    http::{Response},
    response::Redirect,
    Form,
};

use crate::{syntax::serial::populate_graph, router::handlers, types::Node};

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

#[cfg(test)]
mod tests {
    use axum::{
        http::{StatusCode},
    };
    use super::*;

    #[tokio::test]
    async fn search_redirect() {
        let query = Form(Query {
            node: String::from("duZzBrgCzMhVY15wehxasezsGNatOKIq"),
        });
        let response = search(query).await;
        assert!(response.status_code() == StatusCode::PERMANENT_REDIRECT);
    }

    #[tokio::test]
    async fn about_page_ok() {
        let response = page("about.html").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn tree_page_ok() {
        let response = page("tree.html").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn inexistent_page_error() {
        let response = page("HBvcwqT8wLk6hxk1GdvNcEzJ6IiZ2Fod").await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
