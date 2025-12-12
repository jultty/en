use axum::{
    body::Body,
    extract::Path,
    http::{ header, HeaderValue, Response, StatusCode },
    response::{ Redirect },
    routing::get,
    Form,
    Router,
};

use formats::{populate_graph, serialize_graph, Format};
use types::Node;

mod formats;
mod types;

static ONSET: std::sync::LazyLock<std::time::Instant> =
    std::sync::LazyLock::new(std::time::Instant::now);

#[tokio::main]
async fn main() {

    std::panic::set_hook(Box::new(|info| {

        let payload = info.payload_as_str().unwrap_or(
            "No string payload. Is edition > 2021?");

        let location = info.location().map_or_else(
                || "location unavailable".to_string(),
                |s| format!("{}:{}:{}", s.file(), s.line(), s.column()));

        eprintln!(" P! [{:?}] {}: {}", ONSET.elapsed(), location, payload);

    }));

    let app = Router::new()
        .route("/", get(index).post(query))
        .route("/graph/toml", get(toml_graph))
        .route("/graph/json", get(json_graph))
        .route("/static/style.css", get(
            || { file_handler("./static/style.css", "text/css") }))
        .route("/static/favicon.svg", get(
            || { file_handler("./static/favicon.svg", "image/svg+xml") }))
        .route("/node/{node_id}", get(node_view).post(node_view))
        .route("/tree", get(tree))
        .route("/about", get(|| static_template_handler("about.html")))
        .route("/acknowledgments", get(|| {
            static_template_handler("acknowledgments.html")
        }))
        .fallback(not_found)
    ;

    if let Ok(listener) = tokio::net::TcpListener::bind("0.0.0.0:3000").await
        .or(Err("Failed to instantiate Tokio listener")) {

        match axum::serve(listener, app).await {
            Ok(()) => (),
            Err(e) => {
                eprintln!("Failed to serve application with axum::serve: {e:#?}");
                std::process::exit(1);
            },
        }
    }
}

fn make_body(
    name: &str,
    context: &tera::Context,
    error_message: Option<String>,
) -> (String, u16) {

    let tera = match tera::Tera::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"),
    ) {
        Ok(t) => t,
        Err(e) => {
            println!("Tera parsing error: {e:#?}");
            panic!("{e}")
        }
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
                None => {
                    &format!(
                        "Template render failed.\n\
                        Engine message:\n<pre>{e:#?}</pre>\n\
                        Context:\n<pre>{context:#?}</pre>"
                    )
                }
            };

            error_context.insert("message", out_error_message);
            error_context.insert("title",
                &StatusCode::INTERNAL_SERVER_ERROR.to_string());

            (tera.render("error.html", &error_context)
                .unwrap_or(out_error_message.clone()), 500)
        }
    }
}

fn make_response(
    body: &str,
    status_code: u16,
    headers: &[(header::HeaderName, &str)]
) -> Response<Body> {

    let mut response = Response::new(Body::from(body.to_owned()));

    *response.status_mut() = StatusCode::from_u16(status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    for header in headers {
        if let Ok(wrapped) = HeaderValue::from_str(header.1) {
            if let Some(overwritten) = response.headers_mut().insert(
                header.0.clone(),
                wrapped,
            ) { eprintln!("[make_response] Overwrote header {overwritten:?} \
                    because another for key {} already existed", header.0);
            }
        } else {
            eprintln!("[make_response] Failed to wrap header value {}",
                header.1);
        }
    }

    response
}

fn template_handler(
    name: &str,
    context: &tera::Context,
    error_code: u16,
    error_message: Option<String>,
    is_error: bool,
) -> Response<Body> {

    let (body, render_status) = make_body(
        name, context, error_message);

    let status_code = if is_error { error_code } else { render_status };

    make_response(&body, status_code,
        &[(header::CONTENT_TYPE, "text/html")])
}

async fn node_view(Path(id): Path<String>) -> Response<Body>  {

    let mut context = tera::Context::new();

    let graph = populate_graph();
    let nodes = graph.nodes;
    let empty_node = Node::new(
        Some(format!("Could not find node with ID {id}.")),
    );

    let node: &Node = nodes.get(&id).unwrap_or(&empty_node);

    context.insert("id", &id);
    context.insert("title", &node.title);
    context.insert("text", &node.text);
    context.insert("connections", &node.connections.clone());
    context.insert("incoming", &graph.incoming.get(&id));

    let not_found = node.clone() == empty_node;
    let template_name = "node.html".to_string();

    template_handler(
        &template_name,
        &context,
        if not_found { 404 } else { 500 },
        Some(format!(
            "Failed to generate page for node {} (ID {}).\n\
            Node struct: <pre>{:#?}</pre>",
            node.title, id, node
        ).to_owned()),
        not_found,
    )
}

async fn index() -> Response<Body> {

    let mut context = tera::Context::new();
    let graph = populate_graph();
    let root_node = graph.get_root().unwrap_or_default();
    let nodes: Vec<Node> = graph.nodes.into_values().collect();

    context.insert("nodes", &nodes);
    context.insert("root_node", &root_node);

    template_handler("index.html", &context, 500, None, false)
}

async fn tree() -> Response<Body> {

    let mut context = tera::Context::new();
    let graph = populate_graph();
    let root_node = graph.get_root().unwrap_or_default();
    let nodes: Vec<Node> = graph.nodes.into_values().collect();

    context.insert("nodes", &nodes);
    context.insert("root_node", &root_node);

    template_handler("tree.html", &context, 500, None, false)
}

#[expect(clippy::unused_async)]
async fn static_template_handler(name: &str) -> Response<Body> {
    template_handler(name, &tera::Context::new(), 500, None, false)
}

#[expect(clippy::unused_async)]
async fn file_handler(
    file_path: &str,
    content_type: &str,
) -> Response<Body> {

    let content = match std::fs::read(file_path) {
        Ok(s) => s,
        Err(e) => panic!("[static_file_handler] Failed to read file contents: {e}"),
    };

    let mut response = Response::new(Body::from(content));
    *response.status_mut() = StatusCode::OK;
    let header = header::CONTENT_TYPE;

    if let Ok(header_value) = HeaderValue::from_str(content_type) {
        if let Some(h) = response.headers_mut().insert(header, header_value) {
            eprintln!("[static_file_handler] Overwrote existing header {h:?} \
                because a header for the same key existed");
        }
    } else { eprintln!("[static_file_handler] Failed to create content type \
        header value from {content_type}"); }

    response
}

#[derive(serde::Deserialize)]
struct Query { node: String }

async fn query(Form(query): Form<Query>) -> Redirect {
    Redirect::permanent(format!("/node/{}", query.node).as_str())
}

async fn json_graph() -> Response<Body> {
    let graph = populate_graph();
    let body = serialize_graph(&Format::Json, &graph);

    make_response(&body, 200, &[(header::CONTENT_TYPE, "application/json")])
}

async fn toml_graph() -> Response<Body> {
    let graph = populate_graph();
    let body = serialize_graph(&Format::Toml, &graph);

    make_response(&body, 200, &[(header::CONTENT_TYPE, "text/plain")])
}

fn make_error_body(
    code: Option<u16>,
    message: Option<&str>,
) -> String {

    let mut context = tera::Context::new();

    let out_code = code.unwrap_or(500);
    let out_message = &message.unwrap_or("Unknown error");

    context.insert("title", &StatusCode::from_u16(out_code)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR).to_string());
    context.insert("message", out_message);
    context.insert("status_code", &out_code.to_string());

    make_body("error.html", &context, Some(&format!(
        "Failed to render template for Error {out_code}: {out_message}"
    )).cloned()).0
}

fn make_error_response(
    code: Option<u16>,
    message: Option<&str>,
) -> Response<Body> {

    let out_code = code.unwrap_or(500);
    let out_message = &message.unwrap_or("Unknown error");

    let body = make_error_body(Some(out_code), Some(out_message));

    make_response(
        &body,
        out_code,
        &[(header::CONTENT_TYPE, "text/html")],
    )
}

async fn not_found() -> Response<Body> {
    make_error_response(
        Some(404),
        Some("The page you tried to access could not be found."),
    )
}
