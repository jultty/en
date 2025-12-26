use axum::{routing::get, Router};

use crate::{syntax::serial::Format, types::Graph};

mod handlers;

pub fn new(graph: &Graph) -> Router {
    let mut router = Router::new()
        .route(
            "/",
            get(|| handlers::navigation::page("index.html"))
                .post(handlers::navigation::search),
        )
        .route(
            "/static/style.css",
            get(|| handlers::fixed::file("./static/style.css", "text/css")),
        )
        .route(
            "/static/favicon.svg",
            get(|| {
                handlers::fixed::file("./static/favicon.svg", "image/svg+xml")
            }),
        )
        .route(
            "/node/{node_id}",
            get(handlers::graph::node).post(handlers::graph::node),
        )
        .fallback(handlers::error::not_found);

    if graph.meta.config.about {
        router = router
            .route("/about", get(|| handlers::navigation::page("about.html")));
    }

    if graph.meta.config.tree {
        router = router
            .route("/tree", get(|| handlers::navigation::page("tree.html")));
    }

    if graph.meta.config.raw {
        if graph.meta.config.raw_json {
            router = router.route(
                "/graph/json",
                get(|| handlers::fixed::serial(&Format::JSON)),
            );
        }
        if graph.meta.config.raw_toml {
            router = router.route(
                "/graph/toml",
                get(|| handlers::fixed::serial(&Format::TOML)),
            );
        }
    }

    router
}

#[cfg(test)]
mod tests {
    use crate::{
        syntax::serial::populate_graph,
        types::{Config, Meta},
    };

    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
    };
    use tower::ServiceExt as _;

    async fn request(uri: &str, config: Option<&Config>) -> Response<Body> {
        let default_graph = populate_graph();
        let graph = Graph {
            meta: Meta {
                config: config
                    .map(|c| c.to_owned())
                    .unwrap_or(default_graph.meta.config),
                ..default_graph.meta
            },
            ..default_graph
        };
        let router = new(&graph);

        router
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn smoke() {
        let router = axum::Router::new();
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
        let config = Config {
            about: false,
            ..populate_graph().meta.config
        };

        let response = request("/about", Some(&config)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn no_tree_page() {
        let config = Config {
            tree: false,
            ..populate_graph().meta.config
        };

        let response = request("/tree", Some(&config)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn no_toml_raw_graph() {
        let config = Config {
            raw_toml: false,
            ..populate_graph().meta.config
        };

        let response = request("/graph/toml", Some(&config)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn no_json_raw_graph() {
        let config = Config {
            raw_json: false,
            ..populate_graph().meta.config
        };

        let response = request("/graph/json", Some(&config)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn no_raw_graph() {
        let config = Config {
            raw: false,
            ..populate_graph().meta.config
        };

        let toml_response = request("/graph/toml", Some(&config)).await;
        assert_eq!(toml_response.status(), StatusCode::NOT_FOUND);
        let json_response = request("/graph/json", Some(&config)).await;
        assert_eq!(json_response.status(), StatusCode::NOT_FOUND);
    }
}
