use axum::{
    extract::State,
    response::IntoResponse as _,
    {body::Body, extract::Path, http::Response, response::Redirect},
};

use crate::{
    graph::Node,
    prelude::*,
    router::{GlobalState, handlers},
};

pub async fn node(
    Path(id): Path<String>,
    State(state): State<GlobalState>,
) -> Response<Body> {
    let instant = now();
    let result = state.graph.find_node(&id);
    let found = result.node.is_some();
    let node = result
        .node
        .unwrap_or(Node::new(Some(format!("Could not find node ID {id}."))));

    if !node.redirect.is_empty() {
        return Redirect::permanent(
            format!("/node/{}", node.redirect).as_str(),
        )
        .into_response();
    }

    if found && !result.exact {
        return Redirect::permanent(format!("/node/{}", node.id).as_str())
            .into_response();
    }

    let mut context = tera::Context::default();
    context.insert("graph", &state.graph);
    context.insert("node", &node);
    context.insert("incoming", &state.graph.incoming.get(&id));

    tlog!(&instant, "Assembled response for node {}", node.id);
    handlers::template::with_context(
        "node",
        &context,
        if found { 500 } else { 404 },
        Some(
            format!(
                "Failed to generate page for node {} (ID {}).",
                node.title, id
            )
            .to_owned(),
        ),
        !found,
    )
}

#[cfg(test)]
mod tests {
    use axum::{
        http::{HeaderName, StatusCode},
    };

    use crate::graph::Graph;

    use super::*;

    async fn wrap_node(query: &str) -> Response<Body> {
        let state = GlobalState {
            graph: Graph::load(),
        };
        node(Path(query.to_string()), axum::extract::State(state)).await
    }

    #[tokio::test]
    async fn syntax() {
        let response = wrap_node("Syntax").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn syntax_content_type() {
        let response = wrap_node("Syntax").await;
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
        let response = wrap_node("InexistentNode").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn redirect() {
        let response = wrap_node("syntax").await;
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
    }

    #[tokio::test]
    async fn docs_redirect() {
        let response = wrap_node("docs").await;
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
    }
}
