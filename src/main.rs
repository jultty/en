use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{ Html, IntoResponse, Redirect },
    routing::get,
    Form,
    Router,
};

mod types;
mod formats;

use formats::*;
use types::*;

#[tokio::main]
async fn main() {

    let app = Router::new()
        .route("/", get(index).post(query))
        .route("/graph/toml", get(toml_graph))
        .route("/graph/json", get(json_graph))
        .route("/static/style.css", get(stylesheet))
        .route("/node/{node_id}", get(node_view).post(node_view))
        .route("/tree", get(tree))
        .fallback(not_found)
    ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn make_body(
    name: &str,
    context: tera::Context,
    error_code: u16,
    error_message: &str,
) -> String {

    let tera = match tera::Tera::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"),
    ) {
        Ok(t) => t,
        Err(e) => {
            println!("Tera parsing error: {}", e);
            ::std::process::exit(1);
        }
    };

    let render_result = match tera.render(name, &context) {
        Ok(t) => t,
        Err(e) => {

            let mut error_context = tera::Context::new();
            let error = StatusCode::from_u16(error_code)
                .unwrap_or(StatusCode::NOT_IMPLEMENTED);
            error_context.insert("title", &error.to_string());
            error_context.insert(
                "message",
                &format!(
                    r#"<strong>Error while filling template {name}:</strong> {}
                    <strong>User message:</strong> {error_message}"#,
                    e.to_string(),
                ),
            );

            tera.render("error.html", &error_context)
                .unwrap_or(error_message.to_string())
        }
    };

    render_result
}


fn template_handler(
    name: &str,
    context: tera::Context,
    error_code: u16,
    error_message: &str,
) -> Html<String> {
    let body = make_body(name, context, error_code, error_message);
    Html(body)
}

async fn node_view(Path(id): Path<String>) -> impl IntoResponse  {

    let mut context = tera::Context::new();

    let graph = populate_graph();
    let nodes = graph.nodes;
    let empty_node = Node::new(
        Some(format!("Could not find node with ID {}.", id)),
    );

    let node: &Node = nodes.get(&id).unwrap_or(&empty_node);

    context.insert("id", &id);
    context.insert("title", &node.title);
    context.insert("body", &node.body);
    context.insert("connections", &node.connections.clone());
    context.insert("incoming", &graph.incoming.get(&id));

    template_handler(
        "node.html",
        context,
        500,
        &format!(
            r#"Failed to generate page for node {} (ID {}) with {} outgoing,
            {} incoming connections and body "{}""#,
            node.title,
            id,
            node.connections.iter().len(),
            graph.incoming.get(&id).iter().len(),
            node.body,
        ),
    )
}

async fn index() -> Html<String> {

    let mut context = tera::Context::new();
    let graph = populate_graph();
    let root_node = graph.get_root().unwrap_or_default();
    let nodes: Vec<Node> = graph.nodes.into_values().collect();

    context.insert("nodes", &nodes);
    context.insert("root_node", &root_node);

    template_handler("index.html", context, 500, "Failed to render template.")
}

async fn tree() -> Html<String> {

    let mut context = tera::Context::new();
    let graph = populate_graph();
    let root_node = graph.get_root().unwrap_or_default();
    let nodes: Vec<Node> = graph.nodes.into_values().collect();

    context.insert("nodes", &nodes);
    context.insert("root_node", &root_node);

    template_handler("tree.html", context, 500, "Failed to render template")
}

#[derive(serde::Deserialize)]
struct Query { node: String }

async fn query(Form(query): Form<Query>) -> Redirect {
    Redirect::permanent(format!("/node/{}", query.node).as_str())
}

async fn json_graph() -> impl IntoResponse {
    let graph = populate_graph();
    let body = serialize_graph(Format::Json, &graph);

    ([(header::CONTENT_TYPE, "application/json")], body)
}

async fn toml_graph() -> impl IntoResponse {
    let graph = populate_graph();
    let body = serialize_graph(Format::Toml, &graph);

    ([(header::CONTENT_TYPE, "text/plain")], body)
}

async fn stylesheet() -> impl IntoResponse {
    let body = match std::fs::read_to_string("./static/style.css") {
        Ok(s) => s,
        Err(e) => format!("Error: {e}"),
    };

    ([(header::CONTENT_TYPE, "text/css")], body)
}

fn make_error_body(
    code: Option<u16>,
    message: Option<&str>,
) -> String {

    let mut context = tera::Context::new();

    let code = code.unwrap_or(501);
    let message = &message.unwrap_or("Unknown error");

    context.insert("title", &StatusCode::from_u16(code)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR).to_string());
    context.insert("message", message);
    context.insert("status_code", &code.to_string());

    make_body("error.html", context, 500, &format!(
            "Failed to render template for Error {}: {}",
            code,
            message,
        ))
}

fn make_error_response(
    code: Option<u16>,
    message: Option<&str>,
) -> impl IntoResponse {

    let code = code.unwrap_or(501);
    let message = &message.unwrap_or("Unknown error");

    let body = make_error_body(Some(code), Some(message));

    (
        StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        [(header::CONTENT_TYPE, "text/html")],
        body.to_string(),
    )
}

async fn not_found() -> impl IntoResponse {
    make_error_response(
        Some(404),
        Some("The page you tried to access could not be found."),
    )
}
