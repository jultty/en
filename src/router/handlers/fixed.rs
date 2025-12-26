use axum::{
    body::Body,
    http::{Response, StatusCode, header, HeaderValue},
};

use crate::prelude::*;
use crate::{
    router::handlers,
    syntax::serial::{Format, populate_graph, serialize_graph},
};

/// # Panics
/// Will panic if file read fails.
#[expect(clippy::unused_async)]
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
        response.headers_mut().append(header, header_value);
    } else {
        log!("Failed to create content type header value from {content_type}");
    }

    response
}

#[expect(clippy::unused_async)]
pub async fn serial(format: &Format) -> Response<Body> {
    let graph = populate_graph();
    let body = serialize_graph(format, &graph);

    match *format {
        Format::TOML => handlers::raw::make_response(
            &body,
            200,
            &[(header::CONTENT_TYPE, "text/plain")],
        ),
        Format::JSON => handlers::raw::make_response(
            &body,
            200,
            &[(header::CONTENT_TYPE, "application/json")],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn serial_toml() {
        let response = serial(&Format::TOML).await;
        assert!(response.status() == 200);
    }

    #[tokio::test]
    async fn serial_toml_content_type() {
        let response = serial(&Format::TOML).await;
        assert!(
            response.headers().get(header::CONTENT_TYPE).unwrap()
                == "text/plain"
        );
    }

    #[tokio::test]
    async fn serial_json_content_type() {
        let response = serial(&Format::JSON).await;
        assert!(
            response.headers().get(header::CONTENT_TYPE).unwrap()
                == "application/json"
        );
    }

    #[tokio::test]
    async fn file_valid_header() {
        let payload = "y1mgMhjeIMFsRNZ1tskP52DfWuvhvbRP";
        let response = file("./static/graph.toml", payload).await;
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            payload
        );
    }

    #[tokio::test]
    async fn file_invalid_header() {
        let response = file("./static/graph.toml", "\n").await;
        println!("{response:#?}");
        assert!(response.headers().get(header::CONTENT_TYPE).is_none());
    }

    #[tokio::test]
    #[should_panic(
        expected = "Failed to read IvnhZhdHb1xDnUw4hYDDNIERoaOojkiu \
        contents: No such file or directory (os error 2)"
    )]
    async fn file_invalid_path() {
        drop(file("IvnhZhdHb1xDnUw4hYDDNIERoaOojkiu", "text/plain").await);
    }
}
