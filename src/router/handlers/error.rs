use axum::{
    body::Body,
    http::{Response, StatusCode, header},
};

use crate::{syntax::serial::populate_graph, router::handlers};

pub(in crate::router::handlers) fn by_code(
    code: Option<u16>,
    message: Option<&str>,
) -> Response<Body> {
    let out_code = code.unwrap_or(500);
    let out_message = &message.unwrap_or("Unknown error");

    let body = make_body(Some(out_code), Some(out_message));

    handlers::raw::make_response(
        &body,
        out_code,
        &[(header::CONTENT_TYPE, "text/html")],
    )
}

fn make_body(code: Option<u16>, message: Option<&str>) -> String {
    let mut context = tera::Context::new();

    let out_code = code.unwrap_or(500);
    let out_message = &message.unwrap_or("Unknown error");
    let config = populate_graph().meta.config;

    context.insert(
        "title",
        &StatusCode::from_u16(out_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            .to_string(),
    );

    context.insert("message", out_message);
    context.insert("status_code", &out_code.to_string());
    context.insert("config", &config);

    handlers::template::render(
        "error.html",
        &context,
        Some(&format!(
            "Failed to render template for Error {out_code}: {out_message}"
        ))
        .cloned(),
    )
    .0
}

pub async fn not_found() -> Response<Body> {
    by_code(
        Some(404),
        Some("The page you tried to access could not be found."),
    )
}

#[cfg(test)]
mod tests {
    use axum::{
        http::{StatusCode},
    };
    use super::*;

    #[tokio::test]
    async fn not_found() {
        let response = super::not_found().await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn internal_error() {
        assert!(by_code(Some(201), None).status() == 201);
        assert!(by_code(Some(304), None).status() == 304);
        assert!(by_code(Some(418), None).status() == 418);
        assert!(by_code(Some(505), None).status() == 505);
    }

    #[test]
    fn custom_message() {
        let pattern = "sibPtt0mvHPWS9HQ0YBQfGu8cUs954LZ";
        let body = make_body(Some(501), Some(pattern));
        assert!(body.contains(pattern));
        assert!(!body.contains(&pattern.chars().rev().collect::<String>()));
    }
}
