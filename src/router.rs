use axum::{Router, routing::get};

use crate::graph::Graph;

mod handlers {
    pub mod graph;
    pub mod template;
    pub mod raw;
    pub mod navigation;
    pub mod fixed;
    pub mod error;
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
        .route(
            "/static/style.css",
            get(|| handlers::fixed::file("./static/style.css", "text/css")),
        )
        .route(
            "/static/fonts/sans",
            get(|| handlers::fixed::file("./static/fonts/sans", "")),
        )
        .route(
            "/static/fonts/serifed",
            get(|| handlers::fixed::file("./static/fonts/serifed", "")),
        )
        .route(
            "/static/fonts/mono",
            get(|| handlers::fixed::file("./static/fonts/mono", "")),
        )
        .route(
            "/static/fonts/title",
            get(|| handlers::fixed::file("./static/fonts/title", "")),
        )
        .route(
            "/static/fonts/prose",
            get(|| handlers::fixed::file("./static/fonts/prose", "")),
        )
        .route(
            "/static/favicon.svg",
            get(|| {
                handlers::fixed::file("./static/favicon.svg", "image/svg+xml")
            }),
        );

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
mod tests {
    use crate::{
        graph::{Graph, Config, Meta},
    };

    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
    };
    use tower::ServiceExt as _;

    async fn request(uri: &str, config: Option<&Config>) -> Response<Body> {
        let default_graph = Graph::load();
        let graph = Graph {
            meta: Meta {
                config: config
                    .map(std::borrow::ToOwned::to_owned)
                    .unwrap_or(default_graph.meta.config),
                ..default_graph.meta
            },
            ..default_graph
        };
        let router = new(graph);

        router
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn smoke() {
        let router = axum::Router::default();
        let response = router
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn routes() {
        let routes = [
            "/",
            "/about",
            "/tree",
            "/data",
            "/node/Syntax",
            "/static/style.css",
            "/static/favicon.svg",
            "/graph/json",
            "/graph/toml",
        ];

        for route in routes {
            let response = request(route, None).await;
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn no_about_page() {
        let mut config = Config::default();
        config.about = false;

        let response = request("/about", Some(&config)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn no_tree_page() {
        let mut config = Config::default();
        config.tree = false;

        let response = request("/tree", Some(&config)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn no_toml_raw_graph() {
        let mut config = Config::default();
        config.raw_toml = false;

        let response = request("/graph/toml", Some(&config)).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn no_json_raw_graph() {
        let mut config = Config::default();
        config.raw_json = false;

        let response = request("/graph/json", Some(&config)).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn no_raw_graph() {
        let mut config = Config::default();
        config.raw = false;

        let toml_response = request("/graph/toml", Some(&config)).await;
        assert_eq!(toml_response.status(), StatusCode::FORBIDDEN);
        let json_response = request("/graph/json", Some(&config)).await;
        assert_eq!(json_response.status(), StatusCode::FORBIDDEN);
    }
}
