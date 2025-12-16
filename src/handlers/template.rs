use axum::{
    body::Body,
    http::{header, Response, StatusCode},
};

use crate::handlers::raw::make_response;

pub(in crate::handlers) fn by_filename(
    name: &str,
    context: &tera::Context,
    error_code: u16,
    error_message: Option<String>,
    is_error: bool,
) -> Response<Body> {
    let (body, render_status) = render(name, context, error_message);

    let status_code = if is_error { error_code } else { render_status };

    make_response(&body, status_code, &[(header::CONTENT_TYPE, "text/html")])
}

#[expect(clippy::unused_async)]
pub async fn fixed(name: &str) -> Response<Body> {
    by_filename(name, &tera::Context::new(), 500, None, false)
}

pub(in crate::handlers) fn render(
    name: &str,
    // TODO take Option, skip context if None,
    // then template_handler can replace static_template_handler
    context: &tera::Context,
    error_message: Option<String>,
) -> (String, u16) {
    // TODO just return an Option<String> here
    let tera = match tera::Tera::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/**/*"
    )) {
        Ok(t) => t,
        Err(e) => {
            println!("Tera parsing error: {e:#?}");
            panic!("{e}")
        },
    };

    match tera.render(name, context) {
        Ok(t) => (t, 200),
        Err(e) => {
            let mut error_context = tera::Context::new();

            let out_error_message = match error_message {
                Some(s) => &format!(
                    "Template render failed.\n\
                    User message: {s},
                    Engine message:\n<pre>{e:#?}</pre>\n\
                    Context:\n<pre>{context:#?}</pre>"
                ),
                None => &format!(
                    "Template render failed.\n\
                    Engine message:\n<pre>{e:#?}</pre>\n\
                    Context:\n<pre>{context:#?}</pre>"
                ),
            };

            error_context.insert("message", out_error_message);
            error_context.insert(
                "title",
                &StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            );

            (
                tera.render("error.html", &error_context)
                    .unwrap_or(out_error_message.clone()),
                500,
            )
        },
    }
}
