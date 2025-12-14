use axum::{
    body::Body,
    http::{Response, StatusCode, header, HeaderValue},
};

use crate::{
    formats::{Format, populate_graph, serialize_graph},
};
use crate::handlers;

/// # Panics
/// Will panic if file read fails.
pub async fn file(file_path: &str, content_type: &str) -> Response<Body> {
    let content = match std::fs::read(file_path) {
        Ok(s) => s,
        Err(e) => {
            panic!("Failed to read {file_path} contents: {e}")
        },
    };

    let mut response = Response::new(Body::from(content));
    *response.status_mut() = StatusCode::OK;
    let header = header::CONTENT_TYPE;

    if let Ok(header_value) = HeaderValue::from_str(content_type) {
        if let Some(h) = response.headers_mut().insert(header, header_value) {
            crate::dev::log(
                &file,
                &format!(
                    "Overwrote existing header {h:?} because a header for \
                    the same key existed"
                ),
            );
        }
    } else {
        crate::dev::log(
            &file,
            &format!(
                "Failed to create content type \
                header value from {content_type}"
            ),
        );
    }

    response
}

#[expect(clippy::unused_async)]
pub async fn serial(format: &Format) -> Response<Body> {
    let graph = populate_graph();
    let body = serialize_graph(format, &graph);

    match *format {
        Format::Toml => handlers::raw::make_response(
            &body,
            200,
            &[(header::CONTENT_TYPE, "text/plain")],
        ),
        Format::Json => handlers::raw::make_response(
            &body,
            200,
            &[(header::CONTENT_TYPE, "application/json")],
        ),
    }
}
