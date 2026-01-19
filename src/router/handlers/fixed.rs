use axum::{
    http::{HeaderValue, Response, StatusCode, header},
    {
        body::Body,
        extract::{Path, State},
    },
};

use crate::prelude::*;
use crate::{
    graph::{Format, Graph, SerialErrorCause},
    router::{GlobalState, handlers},
    router::handlers::mime::Mime,
};

pub async fn file(
    Path(path): Path<String>,
    State(state): State<GlobalState>,
) -> Response<Body> {
    let instant = now();
    let target = format!("static/public/{path}");
    let content = match std::fs::read(&target) {
        Ok(s) => s,
        Err(e) => {
            let mut error_message = String::from(
                "The requested file does not exist, the server does not have \
                permission to access it or a filesystem error ocurred.",
            );
            if log::env_level() >= DEBUG {
                error_message = format!(
                    "<p>{error_message}</p>\
                    <p>Targeted path: <code>{target}</code></p>\
                    <p>Error message:</p> <pre>{e}</pre>"
                );
            }
            log!(ERROR, "{error_message}");
            return super::error::by_code(
                Some(404),
                Some(&error_message),
                &state.graph,
            );
        },
    };

    let mut response = Response::new(Body::from(content));
    *response.status_mut() = StatusCode::OK;
    let header = header::CONTENT_TYPE;

    let content_type = Mime::guess(&path);

    if let Ok(header_value) =
        HeaderValue::from_str(&String::from(content_type.clone()))
    {
        response.headers_mut().append(header, header_value);
    } else {
        log!(
            WARN,
            "Failed to create content type header value from {content_type:?}"
        );
    }

    tlog!(&instant, "Assembled response for {content_type:?} {path}");
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
    async fn not_found() {
        let state = GlobalState {
            graph: Graph::default(),
        };
        let response = file(Path("/k/j/m".to_string()), State(state)).await;
        assert!(response.status() == StatusCode::NOT_FOUND);
    }
}
