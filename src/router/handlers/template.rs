use std::{collections::HashMap, fs, io::ErrorKind, path::PathBuf};

use axum::{
    body::Body,
    http::{Response, StatusCode, header},
};

use crate::{
    dev::log,
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

static DEFAULTS: &[(&str, &str)] = &[
    ("base.html", include_str!("../../../templates/base.html")),
    ("index.html", include_str!("../../../templates/index.html")),
    ("about.html", include_str!("../../../templates/about.html")),
    ("legal.html", include_str!("../../../templates/legal.html")),
    ("data.html", include_str!("../../../templates/data.html")),
    ("empty.html", include_str!("../../../templates/empty.html")),
    ("error.html", include_str!("../../../templates/error.html")),
    ("node.html", include_str!("../../../templates/node.html")),
    ("tree.html", include_str!("../../../templates/tree.html")),
];

fn read_template(name: &str, path: PathBuf) -> Result<String, std::io::Error> {
    let defaults: HashMap<&str, &str> = DEFAULTS.iter().copied().collect();

    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            match defaults.get(name) {
                Some(default) => Ok(default.to_string()),
                None => Err(error),
            }
        },
        Err(error) => Err(error),
    }
}

fn load_templates() -> Result<tera::Tera, tera::Error> {
    let mut tera = tera::Tera::default();

    let root = PathBuf::from("templates");
    let default_names: Vec<&str> = DEFAULTS.iter().map(|(n, _)| *n).collect();

    log!(
        DEBUG,
        "Reading templates from {}, canonical form {:?}",
        root.display(),
        root.canonicalize()
    );

    match fs::read_dir(&root) {
        Ok(dir) => {
            for file_opt in dir {
                let file = file_opt?;
                let path = file.path();
                if path.is_file() {
                    if let Some(name) = path.clone().file_name() {
                        let Some(name_str) = name.to_str() else {
                            return Err(tera::Error::msg(format!(
                                "Template filename {} is not valid unicode",
                                name.display()
                            )))
                        };
                        if !default_names.contains(&name_str) {
                            tera.add_raw_template(
                                name_str,
                                &read_template(name_str, path)?,
                            )?;
                        }
                    }
                }
            }
        },
        Err(error) => {
            if error.kind() != ErrorKind::NotFound {
                return Err(tera::Error::msg(error.to_string()))
            }
        },
    }

    for tuple in DEFAULTS {
        let path = root.join(tuple.0);
        let name = tuple.0;
        tera.add_raw_template(name, &read_template(name, path)?)?;
    }

    Ok(tera)
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
    let tera = match load_templates() {
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
                Ok(_) => panic!("Got Ok, expected Error"),
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

    #[test]
    fn default_templates_exist_and_match() {
        let templates_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        for (map_name, map_contents) in DEFAULTS {
            let path = templates_dir.join(map_name);
            assert!(path.exists());
            assert!(path.is_file());
            let contents = fs::read_to_string(&path).unwrap();
            assert_eq!(&contents, map_contents);
        }
    }

    #[test]
    fn rendering_error_html_contains_inner_error() {
        let outer_payload = "Gl0c7CyArjlG1Zgvj3D5BFmZT6zRz5Ky";
        let inner_payload = "t53pvXCf0JqUzwiM5BZbYxAQadYSJ9XW";
        let inner_error = tera::Error::msg(inner_payload);
        let error = RenderingError::new(outer_payload, 501, &inner_error);
        assert!(error.template.html.contains(inner_payload));
        assert!(error.template.html.contains(outer_payload));
    }

    #[test]
    fn rendering_error_display() {
        let payload = "4LKNOSqfW0Ys3LALDAond8IIp5RgN7vK";
        let error = RenderingError::new(payload, 501, &tera::Error::msg(""));
        let display_string = format!("{error}");
        assert!(display_string.contains(payload));
    }

    #[test]
    fn empty_template_read_is_an_error() {
        let result = read_template("", PathBuf::from(""));
        assert!(result.is_err());
    }

    #[test]
    fn template_read_without_permissions_is_an_error() {
        let result = read_template("", PathBuf::from("/etc/shadow"));
        assert!(result.is_err());
    }

    #[test]
    fn template_read_without_a_default_is_an_error() {
        let result = read_template(
            "xkQwFZpqf5iz",
            PathBuf::from("templates/Boy5CZQUk2oX"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn template_read_with_a_default_is_ok() {
        let result =
            read_template("base.html", PathBuf::from("templates/St1iFgeOrhCK"));
        assert!(result.is_ok());
    }

    #[test]
    fn template_read_with_a_file_is_ok() {
        let result =
            read_template("GpzjjAPhCTIr", PathBuf::from("templates/base.html"));
        assert!(result.is_ok());
    }
}

#[cfg(test)]
#[expect(clippy::panic_in_result_fn)]
mod serial_tests {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt as _};

    use super::*;
    use crate::dev::test::{Directories, Error};

    #[test]
    #[cfg_attr(not(unix), ignore)]
    fn invalid_utf8_template_filename() -> Result<(), Error> {
        let dirs = Directories::setup("encoding")?;

        let invalid_name = OsStr::from_bytes(&[0xff, 0xfe, 0x80]);
        let file_path = dirs.templates.join(invalid_name);
        fs::write(file_path, b"")?;

        let template_load_result = load_templates();
        let err = template_load_result.err().unwrap();

        let error_message = err.to_string();
        assert!(error_message.contains("not valid unicode"));

        Ok(())
    }

    #[test]
    fn custom_template() -> Result<(), Error> {
        let dirs = Directories::setup("custom_template")?;

        let file_name = "custom.html";
        let file_path = dirs.templates.join(file_name);
        fs::write(file_path, b"")?;

        let engine = load_templates()?;
        assert!(engine.get_template_names().any(|t| t == "custom.html"));

        Ok(())
    }

    #[test]
    fn custom_template_inheritance_error() -> Result<(), Error> {
        let dirs = Directories::setup("custom_template")?;

        let file_name = "custom.html";
        let file_path = dirs.templates.join(file_name);
        fs::write(file_path, br#"{% extends "nonexistent.html" %}"#)?;

        let template_load_result = load_templates();
        assert!(template_load_result.is_err());

        Ok(())
    }

    #[test]
    fn inner_template_no_op() -> Result<(), Error> {
        let dirs = Directories::setup("inner_template")?;

        let inner_dir = dirs.templates.join("inner");
        fs::create_dir(&inner_dir)?;
        let inner_template = inner_dir.join("inner.html");
        fs::write(inner_template, br#"{% extends "nonexistent.html" %}"#)?;

        let engine = load_templates()?;
        let default_count = dirs.original.join("templates").read_dir()?.count();
        let template_count = engine.get_template_names().count();
        assert!(template_count == default_count);

        Ok(())
    }

    #[test]
    fn templates_dir_not_found_ok() -> Result<(), Error> {
        let dirs = Directories::setup("not_found_error")?;

        std::fs::remove_dir_all(&dirs.templates)?;
        let template_load_result = load_templates();
        template_load_result?;

        Ok(())
    }

    #[test]
    // Unexpected here means any error other than 'not found'
    fn templates_dir_unexpected_error() -> Result<(), Error> {
        let dirs = Directories::setup("unexpected_error")?;

        log!(DEBUG, "Working directory is {:?}", std::env::current_dir());

        std::fs::remove_dir_all(&dirs.templates)?;
        let templates = dirs.test.join("templates");
        fs::write(&templates, b"")?;

        let template_load_result = load_templates();
        assert!(template_load_result.is_err());

        Ok(())
    }
}
