use axum::{
    Form, body::Body, extract::State, http::Response, response::Redirect,
};
use serde::Serialize;

use crate::{
    graph::{Graph, Node},
    prelude::*,
    router::{GlobalState, handlers},
};

pub async fn index(State(state): State<GlobalState>) -> Response<Body> {
    handlers::template::with_graph("index", state).await
}

pub async fn about(State(state): State<GlobalState>) -> Response<Body> {
    handlers::template::with_graph("about", state).await
}

pub async fn legal(State(state): State<GlobalState>) -> Response<Body> {
    let mut context = tera::Context::default();
    context.insert("graph", &state.graph);
    context.insert("fonts", &crate::router::handlers::fixed::FONTS);

    handlers::template::with_context("legal", &context, 500, None, false)
}

pub async fn tree(State(state): State<GlobalState>) -> Response<Body> {
    let instant = now();

    let mut context = tera::Context::default();
    context.insert("graph", &state.graph);
    context.insert("tree", &make_tree(&state.graph));

    tlog!(&instant, "Assembled response for tree endpoint");
    handlers::template::with_context("tree", &context, 500, None, false)
}

#[derive(Serialize, Clone)]
struct TreeNode {
    node: Node,
    edges: Vec<Node>,
}

fn make_tree(graph: &Graph) -> Vec<TreeNode> {
    // 'leaf' not in the full graph sense, but this shallow tree representation
    let mut leaf_ids: Vec<String> = vec![];
    let mut sorted_tree: Vec<TreeNode> = vec![];

    // scaffold an ordered vector with all nodes except the root
    for node in graph.nodes.values() {
        if node.id != graph.root_node {
            sorted_tree.push(TreeNode {
                node: node.clone(),
                edges: vec![],
            });
        }
    }

    // sort the vector ascending by those with the most out edges
    sorted_tree.sort_by_key(|pair| pair.node.stats.outgoing);

    // push the root node last in the vector
    if let Some(root_node) = graph.nodes.get(&graph.root_node) {
        sorted_tree.push(TreeNode {
            node: root_node.clone(),
            edges: vec![],
        });
    }

    // reverse vector so it starts with root node and descends by most out edges
    sorted_tree.reverse();

    // push node's outoging connections as its leaves, but only once for each
    // leaf across all nodes
    for pair in &mut sorted_tree {
        for connection_id in pair.node.connections.keys() {
            if !leaf_ids.contains(connection_id)
                && let Some(connection_node) =
                    graph.nodes.get(&connection_id.clone())
            {
                leaf_ids.push(connection_id.clone());
                pair.edges.push(connection_node.clone());
            }
        }
    }

    // drop nodes that are also leaves if they have no out edges
    let mut deduplicated_tree: Vec<TreeNode> = sorted_tree
        .iter()
        .filter(|pair| {
            !(leaf_ids.contains(&pair.node.id) && pair.edges.is_empty())
        })
        .cloned()
        .collect();

    // collect the final top-level node ids
    let deduplicated_tree_nodes: Vec<String> = deduplicated_tree
        .iter()
        .map(|pair| pair.node.id.clone())
        .collect();

    // drop all leaves that are already top-level nodes
    for pair in &mut deduplicated_tree {
        pair.edges = pair
            .edges
            .iter()
            .filter(|connection| {
                !deduplicated_tree_nodes.contains(&connection.id)
            })
            .cloned()
            .collect();
    }

    deduplicated_tree
}

pub async fn data(State(state): State<GlobalState>) -> Response<Body> {
    let instant = now();

    let mut detached_pairs: Vec<(String, u32)> =
        state.graph.stats.detached.clone().into_iter().collect();
    detached_pairs.sort_by_key(|b| std::cmp::Reverse(b.1));

    let mut context = tera::Context::default();
    context.insert("graph", &state.graph);
    context.insert("detached_count", &state.graph.stats.detached.len());
    context.insert("detached_pairs", &detached_pairs);

    tlog!(&instant, "Assembled response for data endpoint");
    handlers::template::with_context("data", &context, 500, None, false)
}

pub async fn search(Form(query): Form<Query>) -> Redirect {
    Redirect::permanent(format!("/node/{}", query.node).as_str())
}

pub async fn redirect(Form(query): Form<Query>) -> Redirect {
    Redirect::permanent(format!("/node/{}", query.node).as_str())
}

#[derive(serde::Deserialize)]
pub struct Query {
    node: String,
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::*;
    use crate::graph::Graph;

    async fn wrap_page(path: &str) -> Response<Body> {
        let state = GlobalState {
            graph: Graph::load(),
        };
        handlers::template::with_graph(path, state).await
    }

    #[tokio::test]
    async fn search_redirect() {
        let query = Form(Query {
            node: String::from("duZzBrgCzMhVY15wehxasezsGNatOKIq"),
        });
        let response = search(query).await;
        assert!(response.status_code() == StatusCode::PERMANENT_REDIRECT);
    }

    #[tokio::test]
    async fn about_page_ok() {
        let response = wrap_page("about").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn tree_page_ok() {
        let state = GlobalState {
            graph: Graph::load(),
        };
        let response = tree(State(state)).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn inexistent_page_error() {
        use tower::ServiceExt as _;

        let payload = "HBvcwqT8wLk6hxk1GdvNcEzJ6IiZ2Fod";
        let response = wrap_page(payload).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let router = axum::Router::default();
        let live_mock_response = router
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/{payload}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(live_mock_response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn id_redirect() {
        let query = Form(Query {
            node: String::from("ancHOr syntaX"),
        });
        let response = search(query).await;
        assert!(response.status_code() == StatusCode::PERMANENT_REDIRECT);
    }

    #[tokio::test]
    async fn dedicated_redirect_endpoint() {
        let query = Form(Query {
            node: String::from("syNTaX"),
        });
        let response = redirect(query).await;
        assert!(response.status_code() == StatusCode::PERMANENT_REDIRECT);
    }

    #[test]
    fn tree_contains_all_nodes_exactly_once() {
        let graph = Graph::load();
        let tree = make_tree(&graph);
        let mut node_ids: Vec<String> = graph.nodes.keys().cloned().collect();
        let mut tree_node_ids: Vec<String> = vec![];
        let mut tree_leaf_ids: Vec<String> = vec![];

        for pair in tree {
            tree_node_ids.push(pair.node.id);
            for edge in pair.edges {
                tree_leaf_ids.push(edge.id);
            }
        }

        node_ids.sort();
        node_ids.dedup();
        let mut tree_ids: Vec<String> =
            [tree_node_ids.clone(), tree_leaf_ids.clone()]
                .iter()
                .flatten()
                .cloned()
                .collect();
        tree_ids.sort();
        tree_node_ids.sort();
        tree_leaf_ids.sort();
        let mut tree_ids_dedup = tree_ids.clone();
        tree_ids_dedup.dedup();
        let mut tree_node_ids_dedup = tree_node_ids.clone();
        tree_node_ids_dedup.dedup();
        let mut tree_leaf_ids_dedup = tree_leaf_ids.clone();
        tree_leaf_ids_dedup.dedup();

        assert_eq!(tree_ids, node_ids);
        assert_eq!(tree_leaf_ids, tree_leaf_ids_dedup);
        assert_eq!(tree_node_ids, tree_node_ids_dedup);
        assert_eq!(tree_ids, tree_ids_dedup);
    }
}
