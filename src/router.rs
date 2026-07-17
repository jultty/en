use axum::{Router, routing::get};

use crate::graph::Graph;

mod handlers {
    mod asset;
    pub mod error;
    pub mod fixed;
    pub mod graph;
    pub mod mime;
    pub mod navigation;
    pub mod raw;
    pub mod template;
}

#[derive(Clone)]
pub struct GlobalState {
    pub graph: Graph,
}

pub fn new(graph: Graph) -> Router {
    let state = GlobalState { graph };

    let mut router = Router::default()
        .route(
            "/",
            get(handlers::navigation::index).post(handlers::navigation::search),
        )
        .route(
            "/node/{node_id}",
            get(handlers::graph::node).post(handlers::graph::node),
        )
        .route("/data", get(handlers::navigation::data))
        .route("/graph/{format}", get(handlers::fixed::serial))
        .route("/search", get(handlers::navigation::search))
        .route("/redirect", get(handlers::navigation::redirect))
        .route("/static/{*path}", get(handlers::fixed::file))
        .route("/legal", get(handlers::navigation::legal));

    if state.graph.meta.config.tree {
        router = router.route("/tree", get(handlers::navigation::tree));
    }
    if state.graph.meta.config.about {
        router = router.route("/about", get(handlers::navigation::about));
    }

    router
        .fallback(handlers::error::not_found)
        .with_state(state)
}

#[cfg(test)]
#[expect(clippy::panic_in_result_fn)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt as _;

    use crate::{
        dev::test::{self, request},
        graph::Graph,
    };

    #[tokio::test]
    async fn smoke() -> Result<(), test::Error> {
        let router = axum::Router::default();
        let response = router
            .oneshot(Request::builder().uri("/").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn routes() -> Result<(), test::Error> {
        let routes = [
            "/",
            "/about",
            "/tree",
            "/data",
            "/node/Syntax",
            "/static/assets/style.css",
            "/static/assets/favicon.svg",
            "/graph/json",
            "/graph/toml",
        ];

        for route in routes {
            let result = request(route, Some(&Graph::load())).await;
            match result {
                Ok(response) => {
                    eprintln!("{route}: {}", response.status());
                    assert_eq!(StatusCode::OK, response.status());
                },
                Err(error) => eprintln!("{error:#?}"),
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn no_about_page() -> Result<(), test::Error> {
        let mut graph = Graph::default();
        graph.meta.config.about = false;

        let response = request("/about", Some(&graph)).await;
        assert_eq!(response?.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn no_tree_page() -> Result<(), test::Error> {
        let mut graph = Graph::default();
        graph.meta.config.tree = false;

        let response = request("/tree", Some(&graph)).await;
        assert_eq!(response?.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn no_toml_raw_graph() -> Result<(), test::Error> {
        let mut graph = Graph::default();
        graph.meta.config.raw_toml = false;

        let response = request("/graph/toml", Some(&graph)).await;
        assert_eq!(response?.status(), StatusCode::FORBIDDEN);

        Ok(())
    }

    #[tokio::test]
    async fn no_json_raw_graph() -> Result<(), test::Error> {
        let mut graph = Graph::default();
        graph.meta.config.raw_json = false;

        let response = request("/graph/json", Some(&graph)).await;
        assert_eq!(response?.status(), StatusCode::FORBIDDEN);

        Ok(())
    }

    #[tokio::test]
    async fn no_raw_graph() -> Result<(), test::Error> {
        let mut graph = Graph::default();
        graph.meta.config.raw = false;

        let toml_response = request("/graph/toml", Some(&graph)).await;
        assert_eq!(toml_response?.status(), StatusCode::FORBIDDEN);
        let json_response = request("/graph/json", Some(&graph)).await;
        assert_eq!(json_response?.status(), StatusCode::FORBIDDEN);

        Ok(())
    }
}
