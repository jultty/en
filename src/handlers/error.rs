use axum::{
    body::Body,
    http::{Response, StatusCode, header},
};

use crate::handlers;

pub(in crate::handlers) fn by_code(
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

    context.insert(
        "title",
        &StatusCode::from_u16(out_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            .to_string(),
    );

    context.insert("message", out_message);
    context.insert("status_code", &out_code.to_string());

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
