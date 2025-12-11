use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Redirect},
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
        .route("/static/style.css", get(stylesheet))
        .route("/static/favicon.svg", get(favicon))
        .route("/node/{node_id}", get(node_view).post(node_view))
        .route("/tree", get(tree))
        .route("/about", get(|| static_template_handler("about.html")))
        .route("/acknowledgments", get(|| static_template_handler("acknowledgments.html")))
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
    error_message: Option<&str>,
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

fn template_handler(
    name: &str,
    context: tera::Context,
    error_code: u16,
    error_message: Option<String>,
    is_error: bool,
) -> impl IntoResponse {

    let (body, render_status) = make_body(
        name, &context, error_message.as_deref());

    let status_code = if render_status != 200 {
        StatusCode::from_u16(render_status)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    } else if is_error {
        StatusCode::from_u16(error_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    } else { StatusCode::OK  };

    (
        status_code,
        [(header::CONTENT_TYPE, "text/html")],
        body.clone(),
    )
}

async fn node_view(Path(id): Path<String>) -> impl IntoResponse  {

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

    template_handler(
        "node.html",
        context,
        if not_found { 404 } else { 500 },
        Some(format!(
            "Failed to generate page for node {} (ID {}).\n\
            Node struct: <pre>{:#?}</pre>",
            node.title, id, node
        ).to_owned()),
        not_found,
    )
}

async fn index() -> impl IntoResponse {

    let mut context = tera::Context::new();
    let graph = populate_graph();
    let root_node = graph.get_root().unwrap_or_default();
    let nodes: Vec<Node> = graph.nodes.into_values().collect();

    context.insert("nodes", &nodes);
    context.insert("root_node", &root_node);

    template_handler("index.html", context.clone(), 500, None, false)
}

async fn tree() -> impl IntoResponse {

    let mut context = tera::Context::new();
    let graph = populate_graph();
    let root_node = graph.get_root().unwrap_or_default();
    let nodes: Vec<Node> = graph.nodes.into_values().collect();

    context.insert("nodes", &nodes);
    context.insert("root_node", &root_node);

    template_handler("tree.html", context, 500, None, false)
}

#[expect(clippy::unused_async)]
async fn static_template_handler(name: &str) -> impl IntoResponse {
    template_handler(name, tera::Context::new(), 500, None, false)
}

#[derive(serde::Deserialize)]
struct Query { node: String }

async fn query(Form(query): Form<Query>) -> Redirect {
    Redirect::permanent(format!("/node/{}", query.node).as_str())
}

async fn json_graph() -> impl IntoResponse {
    let graph = populate_graph();
    let body = serialize_graph(&Format::Json, &graph);

    ([(header::CONTENT_TYPE, "application/json")], body)
}

async fn toml_graph() -> impl IntoResponse {
    let graph = populate_graph();
    let body = serialize_graph(&Format::Toml, &graph);

    ([(header::CONTENT_TYPE, "text/plain")], body)
}

async fn stylesheet() -> impl IntoResponse {
    let content = match std::fs::read_to_string("./static/style.css") {
        Ok(s) => s,
        Err(e) => format!("Error: {e}"),
    };

    ([(header::CONTENT_TYPE, "text/css")], content)
}

async fn favicon() -> impl IntoResponse {
    let content = match std::fs::read("./static/favicon.svg") {
        Ok(b) => b,
        Err(e) => { eprintln!("Error: {e}"); vec![] }
    };

    ([(header::CONTENT_TYPE, "image/svg+xml")], content)
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
    ))).0
}

fn make_error_response(
    code: Option<u16>,
    message: Option<&str>,
) -> impl IntoResponse {

    let out_code = code.unwrap_or(500);
    let out_message = &message.unwrap_or("Unknown error");

    let body = make_error_body(Some(out_code), Some(out_message));

    (
        StatusCode::from_u16(out_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        [(header::CONTENT_TYPE, "text/html")],
        body.clone(),
    )
}

async fn not_found() -> impl IntoResponse {
    make_error_response(
        Some(404),
        Some("The page you tried to access could not be found."),
    )
}
