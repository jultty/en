use axum::{
    Form, body::Body, extract::State, http::Response, response::Redirect,
};

use crate::{
    prelude::*,
    router::{GlobalState, handlers},
};

pub async fn index(State(state): State<GlobalState>) -> Response<Body> {
    handlers::template::with_graph("index", state).await
}

pub async fn about(State(state): State<GlobalState>) -> Response<Body> {
    handlers::template::with_graph("about", state).await
}

pub async fn tree(State(state): State<GlobalState>) -> Response<Body> {
    let instant = now();

    let mut context = tera::Context::default();
    context.insert("graph", &state.graph);
    if let Some(root_node) = state.graph.get_root() {
        context.insert("root_node", &root_node);
    }

    tlog!(&instant, "Assembled response for tree endpoint");
    handlers::template::with_context("tree", &context, 500, None, false)
}

pub async fn data(State(state): State<GlobalState>) -> Response<Body> {
    let instant = now();

    let mut detached_pairs: Vec<(String, u32)> =
        state.graph.stats.detached.clone().into_iter().collect();
    detached_pairs.sort_by(|a, b| b.1.cmp(&a.1));

    let mut context = tera::Context::default();
    context.insert("graph", &state.graph);
    context.insert("detached_count", &state.graph.stats.detached.len());
    context.insert("detached_pairs", &detached_pairs);

    tlog!(&instant, "Assembled response for data endpoint");
    handlers::template::with_context("data", &context, 500, None, false)
}

pub async fn search(Form(query): Form<Query>) -> Redirect {
    Redirect::permanent(format!("/node/{}", query.node).as_str())
}

pub async fn redirect(Form(query): Form<Query>) -> Redirect {
    Redirect::permanent(format!("/node/{}", query.node).as_str())
}

#[derive(serde::Deserialize)]
pub struct Query {
    node: String,
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::*;
    use crate::graph::Graph;

    async fn wrap_page(path: &str) -> Response<Body> {
        let state = GlobalState {
            graph: Graph::load(),
        };
        handlers::template::with_graph(path, state).await
    }

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
        let response = wrap_page("about").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn tree_page_ok() {
        let response = wrap_page("tree").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn inexistent_page_error() {
        let response = wrap_page("HBvcwqT8wLk6hxk1GdvNcEzJ6IiZ2Fod").await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn id_redirect() {
        let query = Form(Query {
            node: String::from("ancHOr syntaX"),
        });
        let response = search(query).await;
        assert!(response.status_code() == StatusCode::PERMANENT_REDIRECT);
    }

    #[tokio::test]
    async fn dedicated_redirect_endpoint() {
        let query = Form(Query {
            node: String::from("syNTaX"),
        });
        let response = redirect(query).await;
        assert!(response.status_code() == StatusCode::PERMANENT_REDIRECT);
    }
}
