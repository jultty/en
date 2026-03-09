use axum::{
    body::Body,
    http::{Response, StatusCode, header},
};

use crate::{
    prelude::*,
    router::{GlobalState, handlers::raw::make_response},
};

/// Assembles a response containing the graph as its only context.
///
/// The template name **must not** contain the extension.
#[expect(clippy::unused_async)]
pub async fn with_graph(template: &str, state: GlobalState) -> Response<Body> {
    let instant = now();
    let mut context = tera::Context::default();
    context.insert("graph", &state.graph);

    tlog!(&instant, "Assembled response for template {template}");
    with_context(template, &context, 500, None, false)
}

/// Assembles a response with a custom context.
///
/// The template name **must not** contain the extension.
pub(in crate::router::handlers) fn with_context(
    name: &str,
    context: &tera::Context,
    error_code: u16,
    error_message: Option<String>,
    is_error: bool,
) -> Response<Body> {
    match render(name, context, error_message) {
        Ok(rendered) => {
            let status_code = if is_error { error_code } else { rendered.code };
            make_response(
                &rendered.html,
                status_code,
                &[(header::CONTENT_TYPE, "text/html")],
            )
        },
        Err(error) => make_response(
            &error.template.html,
            error.template.code,
            &[(header::CONTENT_TYPE, "text/html")],
        ),
    }
}

#[derive(Debug)]
pub struct Rendered {
    pub html: String,
    pub code: u16,
}

impl Rendered {
    fn ok(html: &str) -> Rendered {
        Rendered {
            code: 200,
            html: String::from(html),
        }
    }
}

#[derive(Debug)]
pub struct RenderingError {
    pub message: String,
    pub template: Rendered,
}

impl RenderingError {
    fn new(message: &str, code: u16, error: &tera::Error) -> RenderingError {
        RenderingError {
            message: String::from(message),
            template: Rendered {
                html: emergency_wrap(error, message),
                code,
            },
        }
    }

    fn with_template(
        message: &str,
        code: u16,
        template: &str,
    ) -> RenderingError {
        RenderingError {
            message: String::from(message),
            template: Rendered {
                html: String::from(template),
                code,
            },
        }
    }
}

impl std::fmt::Display for RenderingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Rendering Error: {}", self.message)
    }
}

/// Renders a template into a String and error code.
///
/// The template name **must not** contain the extension (e.g. `.html`).
pub(in crate::router::handlers) fn render(
    template: &str,
    context: &tera::Context,
    error_message: Option<String>,
) -> Result<Rendered, RenderingError> {
    let instant = now();
    let tera = match tera::Tera::new("./templates/**/*") {
        Ok(engine) => engine,
        Err(error) => {
            return Err(RenderingError::new(
                "Failed instantiating template engine",
                500,
                &error,
            ))
        },
    };

    match tera.render(format!("{template}.html").as_str(), context) {
        Ok(html) => {
            tlog!(&instant, "Rendered template {template}");
            Ok(Rendered::ok(&html))
        },
        Err(e) => {
            let mut error_context = tera::Context::default();

            let mut out_error_message = match error_message {
                Some(s) => emergency_wrap(&e, &s),
                None => emergency_wrap(&e, "Template render failed."),
            };

            if log::env_level() >= VERBOSE {
                out_error_message = format!(
                    "{out_error_message}\n\
                    Context:\n<pre>{context:#?}</pre>"
                );
            }

            log!(ERROR, "{out_error_message}");
            error_context.insert("message", &out_error_message);
            error_context.insert(
                "title",
                &StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            );

            match tera.render("error.html", &error_context) {
                Ok(rendered_error) => Err(RenderingError::with_template(
                    &out_error_message,
                    500,
                    &rendered_error,
                )),
                Err(error_rendering_error) => Err(RenderingError::new(
                    &format!(
                        "Failed to render an error message template for \
                            \"{out_error_message}\""
                    ),
                    500,
                    &error_rendering_error,
                )),
            }
        },
    }
}

fn emergency_wrap(error: &tera::Error, message: &str) -> String {
    log!(ERROR, "{error:#?}");

    let message_element = format!("<p>{message}</p>");

    format!(
        "<!DOCTYPE html>\n\
        <html>\n\
        <head>\n\
            <title>en Pre-Templating Error</title>\n
            <meta \
                http-equiv=\"Content-Type\" \
                content=\"text/html; charset=utf-8\" >\n\
            <meta name=\"viewport\" \
                content=\"width=device-width, initial-scale=1\">\n\
            <style>\n\
                :root {{ color-scheme: light dark; }}\n\
                * {{\n\
                    background: light-dark(#eee, #222);\n\
                    color: light-dark(#000, #f1e9e5);\n\
                    line-height: 1.6em;
                }}\n\
                pre {{ overflow: auto; }}\n\
            </style>\n\
        </head>\n\
        <body>\n\
            <h2><strong>en Early Pre-Templating Error</strong></h2>\n\
            {message_element}\n\
            <pre>\n\
            {error:#?}\n\
            </pre>\n\
            <p>This error may indicate a malformed or missing template.</p>\n\
            <p>If you haven't modified templates, please consider \
                <a href=\"https://codeberg.org/jutty/en/issues\">\
                    reporting it</a>, including:\
            </p>\n\
            <ul>\n\
                <li>The error message above</li>\n\
                <li>en version: <code>{}</code></li>\n\
                <li>If possible, your graph file's <code>[meta.config]</code> \
                values and definition for this page.</li>\n\
            </ul>\n\
        </body>\n\
        </html>\n\
    ",
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn by_filename_forced_error() {
        let response =
            with_context("index", &tera::Context::default(), 418, None, true);
        assert_eq!(response.status(), 418);
    }

    #[test]
    fn by_filename_index() {
        let response =
            with_context("index", &tera::Context::default(), 418, None, false);
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn by_filename_file_not_found() {
        let response = with_context(
            "bwbl3BnWsluIgbO2NV9t3vtihwcjuF6t",
            &tera::Context::default(),
            418,
            None,
            false,
        );
        assert_eq!(response.status(), 500);
    }

    #[test]
    fn by_filename_empty() {
        let response =
            with_context("", &tera::Context::default(), 418, None, false);
        assert_eq!(response.status(), 500);
    }

    #[test]
    fn render_with_context() {
        let payload = "dBgIw8DnNHxJojiXzu445qUC4UpxwZCy";
        let mut context = tera::Context::default();
        let node = crate::graph::Node::not_found(Some(payload.to_string()));
        let graph = Graph::load();
        context.insert("node", &node);
        context.insert("graph", &graph);
        context.insert("incoming", &graph.incoming.get(&node.id));
        match render("node", &context, None) {
            Ok(rendered) => {
                assert_eq!(rendered.code, 200);
                assert!(rendered.html.matches(payload).count() == 1);
            },
            Err(error) => {
                panic!("Errored on template generation with {error:?}")
            },
        }
    }

    #[test]
    fn render_custom_error_message() {
        let payload = "dBgIw8DnNHxJojiXzu445qUC4UpxwZCy";
        match render(
            "ObH9jYUl4wMhUNcXnuqwVVzHoqx4ufyN",
            &tera::Context::default(),
            Some(payload.to_string()),
        ) {
            Ok(_) => panic!("Got Ok, expected Error"),
            Err(error) => {
                assert_eq!(error.template.code, 500);
                assert!(error.template.html.matches(payload).count() == 1);
            },
        }
    }

    #[test]
    fn render_empty() {
        match render(
            "R8D1pxwHZDxcH5SMjR7rZEnIzmpkiHkH",
            &tera::Context::default(),
            None,
        ) {
            Ok(_) => panic!("Got Ok, expected Error"),
            Err(error) => {
                assert_eq!(error.template.code, 500);
                assert!(
                    error
                        .template
                        .html
                        .matches("Template render failed")
                        .count()
                        == 1
                );
            },
        }
    }

    #[test]
    fn render_not_found() {
        let payload = "OL6kb9qHe7Iwr7wFIRKUTeFhF34BRsQo";
        let (body, status) =
            match render(payload, &tera::Context::default(), None) {
                Ok(_) => panic!("Got Ok, expected Error"),
                Err(error) => (error.template.html, error.template.code),
            };

        assert!(body.matches("TemplateNotFound").count() > 0);
        assert!(body.matches(payload).count() > 0);
        assert_eq!(status, 500);
    }

    #[test]
    fn render_bad_context() {
        let (body, status) =
            match render("node", &tera::Context::default(), None) {
                Ok(rendered) => panic!("Got Ok, expected Error"),
                Err(error) => (error.template.html, error.template.code),
            };
        assert!(body.matches("Template render failed.").count() > 0);
        assert_eq!(status, 500);
    }

    #[test]
    fn emergency_wrap_custom_message() {
        let payload = "JLaTtsnd2IFukIOvqFNymeuiaS6nMaUc";
        let error = tera::Error::msg(payload);
        let html = emergency_wrap(&error, "");
        assert!(html.matches(payload).count() == 1);
    }
}
