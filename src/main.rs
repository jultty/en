use std::{backtrace, env, io, panic, sync, time};

use axum::{routing::get, Router};

use formats::Format;

mod formats;
mod types;
mod handlers;
mod dev;

static ONSET: sync::LazyLock<time::Instant> =
    sync::LazyLock::new(time::Instant::now);

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let default_address = "0.0.0.0:0".to_string();
    let address: &String = args.get(1).unwrap_or(&default_address);

    panic::set_hook(Box::new(|info| {
        let payload = info
            .payload_as_str()
            .unwrap_or("No string payload. Is edition > 2021?");

        let location = info.location().map_or_else(
            || "location unavailable".to_string(),
            |s| format!("{}:{}:{}", s.file(), s.line(), s.column()),
        );

        eprintln!(" P! [{:?}] {location}: {payload}", ONSET.elapsed());

        let trace = backtrace::Backtrace::capture();
        if trace.status() == backtrace::BacktraceStatus::Captured {
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

    let listener =
        tokio::net::TcpListener::bind(address).await.map_err(|e| {
            dev::log(
                &main,
                &format!("Failed to create listener at {address}: {e:#?}"),
            );
            e
        })?;

    dev::log(
        &main,
        &format!(
            "Listening on {}",
            listener
                .local_addr()
                .map(|s| s.to_string())
                .unwrap_or("<unknown>".to_string())
        ),
    );

    axum::serve(listener, app).await.map_err(|e| {
        dev::log(&main, &format!("Failed to serve application: {e:#?}"));
        io::Error::other(e)
    })?;

    Ok(())
}
