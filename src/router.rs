use axum::{routing::get, Router};

use crate::{handlers, formats::Format};

pub fn new() -> Router {
    Router::new()
        .route(
            "/",
            get(|| handlers::navigation::nexus("index.html"))
                .post(handlers::navigation::search),
        )
        .route(
            "/graph/toml",
            get(|| handlers::fixed::serial(&Format::Toml)),
        )
        .route(
            "/graph/json",
            get(|| handlers::fixed::serial(&Format::Json)),
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
        .route("/tree", get(|| handlers::navigation::nexus("tree.html")))
        .route("/about", get(|| handlers::template::fixed("about.html")))
        .route(
            "/acknowledgments",
            get(|| handlers::template::fixed("acknowledgments.html")),
        )
        .fallback(handlers::error::not_found)
}
