use axum::response::IntoResponse as _;
use axum::{body::Body, extract::Path, http::Response, response::Redirect};

use crate::syntax::content;

use crate::{syntax::serial::populate_graph, router::handlers, types::Node};

pub async fn node(Path(id): Path<String>) -> Response<Body> {
    let graph = populate_graph();
    let result = graph.find_node(&id);
    let nodes: Vec<Node> = graph.nodes.into_values().collect();
    let not_found = result.node.is_none();
    let node = result
        .node
        .unwrap_or(Node::new(Some(format!("Could not find node ID {id}."))));

    if !node.redirect.is_empty() {
        return Redirect::permanent(
            format!("/node/{}", node.redirect).as_str(),
        )
        .into_response();
    }

    if result.redirect {
        return Redirect::permanent(format!("/node/{}", node.id).as_str())
            .into_response();
    }

    let mut context = tera::Context::default();
    context.insert("node", &node);
    context.insert("nodes", &nodes);
    context.insert("text", &content::parse(&node.text, &graph.meta.config));
    context.insert("incoming", &graph.incoming.get(&id));
    context.insert("config", &graph.meta.config);

    handlers::template::by_filename(
        "node.html",
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

#[cfg(test)]
mod tests {
    use axum::{
        http::{HeaderName, StatusCode},
    };

    use super::*;

    #[tokio::test]
    async fn syntax() {
        let response = node(Path("Syntax".to_string())).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn syntax_content_type() {
        let response = node(Path("Syntax".to_string())).await;
        assert!(
            response
                .headers()
                .get(HeaderName::from_static("content-type"),)
                .unwrap()
                .to_str()
                .unwrap()
                == "text/html"
        );
    }

    #[tokio::test]
    async fn not_found() {
        let response = node(Path("InexistentNode".to_string())).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn redirect() {
        let response = node(Path("syntax".to_string())).await;
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
    }
}
