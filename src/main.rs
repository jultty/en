use std::{backtrace, io, panic};

use en::{ONSET, syntax, dev};

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = syntax::arguments::Arguments::new().parse();
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

    let app = en::router::new();

    let listener =
        tokio::net::TcpListener::bind(&address).await.map_err(|e| {
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

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let e = true;
        assert!(e);
    }
}
