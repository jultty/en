use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, Response, StatusCode, header},
};

use crate::prelude::*;
use crate::{
    graph::{Format, Graph, SerialErrorCause},
    router::{GlobalState, handlers},
};

/// # Panics
/// Will panic if file read fails.
#[expect(clippy::unused_async)]
pub async fn file(file_path: &str, content_type: &str) -> Response<Body> {
    let instant = now();
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
        log!(
            WARN,
            "Failed to create content type header value from {content_type}"
        );
    }

    tlog!(
        &instant,
        "Assembled response for {content_type} {file_path}"
    );
    response
}

pub async fn serial(
    Path(format): Path<String>,
    State(state): State<GlobalState>,
) -> Response<Body> {
    let config = &state.graph.meta.config;

    let make_error = |code: u16, message: &str| -> Response<Body> {
        handlers::error::by_code(
            Some(code),
            Some(
                format!(
                    "<p>{message}</p>\n\
            <p>Check the <a href=/data>data</a> \n\
            page for the available formats.</p>"
                )
                .as_str(),
            ),
            &state.graph,
        )
    };

    let forbidden_response =
        make_error(403, "This graph format is not available.");
    let unsupported_response =
        make_error(400, "This graph format is not supported.");
    let parse_failure = make_error(505, "The graph has failed to parse.");

    let body =
        match Graph::to_serial(&state.graph, &Format::from(format.as_str())) {
            Ok(serial) => serial,
            Err(error) => match error.cause {
                SerialErrorCause::MalformedInput => return parse_failure,
                SerialErrorCause::UnsupportedFormat => {
                    return unsupported_response;
                },
            },
        };

    match Format::from(format.as_str()) {
        Format::TOML => {
            if config.raw && config.raw_toml {
                handlers::raw::make_response(
                    &body,
                    200,
                    &[(header::CONTENT_TYPE, "text/plain")],
                )
            } else {
                forbidden_response
            }
        },
        Format::JSON => {
            if config.raw && config.raw_json {
                handlers::raw::make_response(
                    &body,
                    200,
                    &[(header::CONTENT_TYPE, "application/json")],
                )
            } else {
                forbidden_response
            }
        },
        Format::Unsupported => unsupported_response,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn wrap_serial(format: &str) -> Response<Body> {
        let state = GlobalState {
            graph: Graph::load(),
        };
        serial(Path(format.to_string()), State(state)).await
    }

    #[tokio::test]
    async fn serial_toml() {
        let response = wrap_serial("toml").await;
        assert!(response.status() == 200);
    }

    #[tokio::test]
    async fn serial_json() {
        let response = wrap_serial("json").await;
        assert!(response.status() == 200);
    }

    #[tokio::test]
    async fn serial_toml_content_type() {
        let response = wrap_serial("TOML").await;
        assert!(
            response.headers().get(header::CONTENT_TYPE).unwrap()
                == "text/plain"
        );
    }

    #[tokio::test]
    async fn serial_json_content_type() {
        let response = wrap_serial("json").await;
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
