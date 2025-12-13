use axum::{routing::get, Router};

use formats::Format;

mod formats;
mod types;
mod handlers;
mod dev;

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

        eprintln!(" P! [{:?}] {location}: {payload}", ONSET.elapsed());

        let trace = std::backtrace::Backtrace::capture();
        if trace.status() == std::backtrace::BacktraceStatus::Captured {
            eprintln!("\n  Stack trace:\n{trace:#?}");
        }
    }));

    let app = Router::new()
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
        .fallback(handlers::error::not_found);

    if let Ok(listener) = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .or(Err("Failed to instantiate Tokio listener"))
    {
        match axum::serve(listener, app).await {
            Ok(()) => (),
            Err(e) => {
                dev::log(
                    &main,
                    &format!(
                        "Failed to serve application with axum::serve: {e:#?}"
                    ),
                );
                std::process::exit(1);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fail() {
        assert_eq!(0_i32, 1_i32);
    }
}
