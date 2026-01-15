use std::{backtrace, io, panic};

use en::{prelude::*, log, ONSET, graph::Graph, syntax};

#[tokio::main]
#[allow(clippy::print_stderr, clippy::print_stdout)]
async fn main() -> io::Result<()> {
    log::print_state();
    let mut instant = now();

    let args = syntax::command::Arguments::default().parse();
    let address = args.make_address();
    instant = tlog!(&instant, "Parsed CLI arguments");

    panic::set_hook(Box::new(|info| {
        let payload = info
            .payload_as_str()
            .unwrap_or("No string payload. Is edition > 2021?");

        let location = info.location().map_or_else(
            || "location unavailable".to_string(),
            |s| format!("{}:{}:{}", s.file(), s.line(), s.column()),
        );

        let level: u8 = std::env::var("RUST_BACKTRACE")
            .unwrap_or("0".to_string())
            .trim()
            .parse()
            .unwrap_or(0);

        eprintln!(" P! [{:?}] {location}: {payload}", ONSET.elapsed());

        let trace = backtrace::Backtrace::capture();
        if trace.status() == backtrace::BacktraceStatus::Captured && level > 1 {
            eprintln!("\n  Stack trace:\n{trace:#?}");
        }
    }));
    instant = tlog!(&instant, "Set up panic hook");

    let graph = Graph::load();
    instant = tlog!(&instant, "Loaded graph");

    let router = en::router::new(&graph);
    tlog!(&instant, "Initialized router");

    let listener =
        tokio::net::TcpListener::bind(&address).await.map_err(|e| {
            log!(ERROR, "Failed to create listener at {address}: {e:#?}");
            e
        })?;
    tlog!(&instant, "Initialized listener");

    println!(
        "Listening on {}",
        listener
            .local_addr()
            .map(|s| s.to_string())
            .unwrap_or("<unknown>".to_string())
    );

    axum::serve(listener, router).await.map_err(|e| {
        log!(ERROR, "Failed to serve application: {e:#?}");
        io::Error::other(e)
    })?;

    Ok(())
}
