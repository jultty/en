use axum::{
    body::Body,
    http::{Response},
    response::Redirect,
    Form,
};

use crate::{
    graph::{Graph, Node},
    router::handlers,
};

#[expect(clippy::unused_async)]
pub async fn page(template: &str) -> Response<Body> {
    let mut context = tera::Context::default();
    let graph = Graph::load();

    context.insert("graph", &graph);

    handlers::template::by_filename(template, &context, 500, None, false)
}

pub async fn data() -> Response<Body> {
    let mut context = tera::Context::default();
    let graph = Graph::load();

    let mut detached_pairs: Vec<(String, u32)> =
        graph.stats.detached.clone().into_iter().collect();
    detached_pairs.sort_by(|a, b| b.1.cmp(&a.1));

    context.insert("graph", &graph);
    context.insert("detached_count", &graph.stats.detached.len());
    context.insert("detached_pairs", &detached_pairs);

    handlers::template::by_filename("data.html", &context, 500, None, false)
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
