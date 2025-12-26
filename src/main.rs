use std::{backtrace, io, panic};

use en::{prelude::*, ONSET, syntax::serial::populate_graph, syntax};

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = syntax::command::Arguments::new().parse();
    let address = args.make_address();

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

    let graph = populate_graph();
    let router = en::router::new(&graph);

    let listener =
        tokio::net::TcpListener::bind(&address).await.map_err(|e| {
            log!("Failed to create listener at {address}: {e:#?}");
            e
        })?;

    log!(
        "Listening on {}",
        listener
            .local_addr()
            .map(|s| s.to_string())
            .unwrap_or("<unknown>".to_string())
    );

    axum::serve(listener, router).await.map_err(|e| {
        log!("Failed to serve application: {e:#?}");
        io::Error::other(e)
    })?;

    Ok(())
}
