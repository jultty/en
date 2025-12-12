use axum::{routing::get, Router};

use handlers::{
    graph::node,
    navigation::{nexus, search},
    fixed::{file, serial},
    template::static_template_handler,
    error::not_found,
};
use formats::Format;

mod formats;
mod types;
mod handlers;

static ONSET: std::sync::LazyLock<std::time::Instant> =
    std::sync::LazyLock::new(std::time::Instant::now);

#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(|info| {
        let payload = info
            .payload_as_str()
            .unwrap_or("No string payload. Is edition > 2021?");

        let location = info.location().map_or_else(
            || "location unavailable".to_string(),
            |s| format!("{}:{}:{}", s.file(), s.line(), s.column()),
        );

        eprintln!(" P! [{:?}] {}: {}", ONSET.elapsed(), location, payload);
    }));

    let app = Router::new()
        .route("/", get(|| nexus("index.html")).post(search))
        .route("/graph/toml", get(|| serial(&Format::Toml)))
        .route("/graph/json", get(|| serial(&Format::Json)))
        .route(
            "/static/style.css",
            get(|| file("./static/style.css", "text/css")),
        )
        .route(
            "/static/favicon.svg",
            get(|| file("./static/favicon.svg", "image/svg+xml")),
        )
        .route("/node/{node_id}", get(node).post(node))
        .route("/tree", get(|| nexus("tree.html")))
        .route("/about", get(|| static_template_handler("about.html")))
        .route(
            "/acknowledgments",
            get(|| static_template_handler("acknowledgments.html")),
        )
        .fallback(not_found);

    if let Ok(listener) = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .or(Err("Failed to instantiate Tokio listener"))
    {
        match axum::serve(listener, app).await {
            Ok(()) => (),
            Err(e) => {
                eprintln!(
                    "Failed to serve application with axum::serve: {e:#?}"
                );
                std::process::exit(1);
            },
        }
    }
}
