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
