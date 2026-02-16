use std::{collections::HashMap, path::PathBuf};

pub use edge::Edge;
pub use meta::{Config, Meta};
pub use node::Node;
use serde::{Deserialize, Serialize};

use crate::{
    prelude::*,
    syntax::{
        command::Arguments,
        content::{
            self,
            parser::{Token, flatten, token::Anchor},
        },
    },
};

pub mod edge;
pub mod meta;
pub mod node;

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Graph {
    #[serde(default)]
    pub nodes: HashMap<String, Node>,
    #[serde(default)]
    pub root_node: String,
    #[serde(skip_deserializing)]
    pub incoming: HashMap<String, Vec<Edge>>,
    #[serde(skip_deserializing)]
    pub lowercase_keymap: HashMap<String, String>,
    #[serde(default)]
    pub meta: Meta,
    #[serde(default)]
    pub stats: Stats,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Stats {
    pub detached: HashMap<String, u32>,
    pub detached_total: u32,
}

impl Graph {
    pub fn with_message(message: &str) -> Graph {
        let graph = Graph::default();
        let mut messages = graph.meta.messages;
        messages.push(message.to_string());
        Graph {
            meta: Meta {
                messages,
                ..graph.meta
            },
            ..graph
        }
    }

    pub fn malformed(message: Option<&str>) -> Graph {
        let mut graph = if let Some(m) = message {
            Graph::with_message(m)
        } else {
            Graph::default()
        };
        graph.meta.malformed = true;
        graph
    }

    /// Loads a Graph TOML file from CLI arguments or their defaults and
    /// returns a modulated Graph.
    ///
    /// Returns a graph with an error message if any errors are propagated.
    pub fn load() -> Graph {
        let result = Graph::load_file(None);
        match result {
            Ok(graph) => graph,
            Err(error) => Graph::malformed(Some(&error)),
        }
    }

    /// Takes a file path to a TOML file and returns a modulated Graph
    ///
    /// If `path` is None, it will fallback to CLI arguments or their defaults.
    ///
    /// # Errors
    /// Propagates errors from `Graph::read_file`.
    pub fn load_file(path: Option<&str>) -> Result<Graph, String> {
        let mut graph = Graph::from_file(path)?;
        graph.modulate();
        Ok(graph)
    }

    ///  Reads a TOML file into a Graph without modulating it.
    ///
    /// # Errors
    /// Returns Err if it can't read the contents of `in_path`.
    /// Propagates errors from `Graph::from_serial`.
    pub fn from_file(in_path: Option<&str>) -> Result<Graph, String> {
        let cli_path = Arguments::default().parse().graph_path;
        let path = in_path.map_or(cli_path, PathBuf::from);

        let toml_source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                log!(ERROR, "Failed reading {e}");
                return Err("Failed reading file at {path}".to_string());
            },
        };

        let result = Graph::from_serial(&toml_source, &Format::TOML)?;
        Ok(result)
    }

    /// Deserializes the given format into a graph.
    ///
    /// # Errors
    /// Errors on unsupported formats.
    /// Propagates serialization errors.
    pub fn from_serial(
        serial: &str,
        format: &Format,
    ) -> Result<Graph, SerialError> {
        match *format {
            Format::TOML => match toml::from_str::<Graph>(serial) {
                Ok(graph) => Ok(graph),
                Err(error) => Err(SerialError {
                    cause: SerialErrorCause::MalformedInput,
                    message: error.to_string(),
                }),
            },
            Format::JSON => match serde_json::from_str::<Graph>(serial) {
                Ok(graph) => Ok(graph),
                Err(error) => Err(SerialError {
                    cause: SerialErrorCause::MalformedInput,
                    message: error.to_string(),
                }),
            },
            Format::Unsupported => Err(SerialError {
                cause: SerialErrorCause::UnsupportedFormat,
                message: "Unsupported format".to_string(),
            }),
        }
    }

    /// Serializes a graph to the given format.
    ///
    /// # Errors
    /// Errors on unsupported formats.
    /// Propagates serialization errors.
    pub fn to_serial(&self, format: &Format) -> Result<String, SerialError> {
        match *format {
            Format::TOML => match toml::to_string(self) {
                Ok(s) => Ok(s),
                Err(e) => Err(SerialError {
                    cause: SerialErrorCause::MalformedInput,
                    message: e.to_string(),
                }),
            },
            Format::JSON => match serde_json::to_string(self) {
                Ok(s) => Ok(s),
                Err(e) => Err(SerialError {
                    cause: SerialErrorCause::MalformedInput,
                    message: e.to_string(),
                }),
            },
            Format::Unsupported => Err(SerialError {
                cause: SerialErrorCause::UnsupportedFormat,
                message: "Unsupported format".to_string(),
            }),
        }
    }

    fn gather_stats(&mut self) {
        let detached = self.stats.detached.values();
        self.stats.detached_total = detached.sum();
    }

    pub fn modulate(&mut self) {
        let mut instant = now();
        instant = tlog!(&instant, "Started node modulation");
        self.map_lowercase_keys();
        instant = tlog!(&instant, "Mapped lowercase keys");
        self.modulate_nodes();
        instant = tlog!(&instant, "Modulated nodes");
        self.modulate_edges();
        instant = tlog!(&instant, "Modulated edges");
        self.map_incoming();
        instant = tlog!(&instant, "Mapped incoming edges");
        self.gather_stats();
        instant = tlog!(&instant, "Gathered stats");
        self.parse_config();
        tlog!(&instant, "Parsed configuration");
    }

    /// Construct a `HashMap` with incoming connections (reversed edges)
    fn map_incoming(&mut self) {
        for node in self.nodes.clone().into_values() {
            for edge in node.connections.clone().values() {
                let mut edges = self
                    .incoming
                    .get(&edge.to.clone())
                    .unwrap_or(&vec![])
                    .clone();
                edges.extend_from_slice(std::slice::from_ref(edge));
                self.incoming.insert(edge.to.clone(), edges.clone());
            }
        }
    }

    /// Modulates nodes that have been deserialized and are still unprocessed.
    ///
    /// Performs checked arithmetic to the following effect:
    /// - Stats will saturate at `u32::MAX` (`increment_detached` calls)
    fn modulate_nodes(&mut self) {
        let in_nodes = self.nodes.clone();

        for (key, node) in in_nodes.clone() {
            let connections = node.connections.clone();
            let mut new_edges = connections.clone();

            // Modulate connections
            for (connection_key, edge) in connections {
                let mut new_edge = edge.clone();

                // Populate empty "to" and "from" IDs
                if edge.from.is_empty() {
                    new_edge.from.clone_from(&key);
                }

                if edge.to.is_empty() {
                    new_edge.to.clone_from(&connection_key);
                }

                // Flag detached edges
                if (!edge.to.is_empty() && in_nodes.contains_key(&edge.to))
                    || (edge.to.is_empty()
                        && in_nodes.contains_key(&new_edge.to))
                {
                    new_edge.detached = false;
                } else {
                    new_edge.detached = true;
                    self.increment_detached(&edge.to);
                }

                if let Some(e) = new_edges.get_mut(&connection_key) {
                    *e = new_edge;
                }
            }

            // Create connections for each link
            for link in &node.links {
                let detached = !in_nodes.contains_key(link);
                new_edges.insert(
                    link.clone(),
                    Edge {
                        from: key.clone(),
                        to: link.clone(),
                        detached,
                    },
                );

                if detached {
                    self.increment_detached(link);
                }
            }

            // Populate empty titles with IDs
            let new_title = if node.title.is_empty() {
                key.clone()
            } else {
                node.title.clone()
            };

            // Populate empty summaries with the leading part of the node text
            let summary = if node.summary.is_empty() {
                let first_line = if let Some(first) =
                    node.text.split("\n\n").find(|s| !s.is_empty())
                {
                    String::from(first)
                } else {
                    node.text.clone()
                };

                let mut candidate =
                    if let Some(dot_split) = first_line.split_once('.') {
                        format!("{}.", dot_split.0)
                    } else {
                        first_line
                    };

                if candidate.len() > 300 {
                    candidate.truncate(300);
                    candidate.push('…');
                }
                candidate
            } else {
                node.summary.clone()
            };

            // Assemble new node
            let new_node = Node {
                id: key.clone(),
                title: new_title,
                summary: flatten(&summary, self),
                connections: new_edges,
                ..node.clone()
            };

            self.nodes.insert(key.clone(), new_node);
        }
    }

    /// Modulates edges that have been deserialized and are still unprocessed.
    ///
    /// Performs checked arithmetic to the following effect:
    /// - Stats will saturate at `u32::MAX`
    fn modulate_edges(&mut self) {
        let graph = self.clone();
        let iterator = self.nodes.iter_mut();
        for (key, node) in iterator {
            // Parse node text
            let parse_output = content::rich_parse(&node.text, &graph);
            node.text
                .clone_from(&parse_output.text.clone().unwrap_or_default());

            // Create connections for each anchor
            let parsed_anchors =
                parse_output.only(&Token::Anchor(Box::default()));
            let mut anchors: Vec<Anchor> = vec![];
            for token in parsed_anchors {
                if let Token::Anchor(token_data) = token {
                    anchors.push(*token_data.clone());
                }
            }

            for anchor in anchors {
                if let Some(anchor_node) = anchor.node() {
                    node.connections.insert(
                        anchor_node.id.clone(),
                        Edge {
                            from: key.clone(),
                            to: anchor_node.id,
                            detached: false,
                        },
                    );
                } else {
                    if let Some(destination) = anchor.destination()
                        && !anchor.external()
                    {
                        let trimmed_destination = destination
                            .trim_start_matches("/node/")
                            .to_string();
                        node.connections.insert(
                            trimmed_destination.clone(),
                            Edge {
                                from: key.clone(),
                                to: trimmed_destination.clone(),
                                detached: true,
                            },
                        );

                        self.stats
                            .detached
                            .entry(trimmed_destination)
                            .and_modify(|count| {
                                *count = count.saturating_add(1);
                            })
                            .or_insert(1);
                    }
                }
            }
        }
    }

    /// Increments detached node statistics for the given node ID
    ///
    /// Performs checked arithmetic to the following effect:
    /// - Stats will saturate at `u32::MAX`
    fn increment_detached(&mut self, node_id: &str) {
        self.stats
            .detached
            .entry(node_id.to_string())
            .and_modify(|count| *count = count.saturating_add(1))
            .or_insert(1);
    }

    pub fn map_lowercase_keys(&mut self) {
        for key in self.nodes.keys() {
            self.lowercase_keymap
                .insert(key.clone().to_lowercase(), key.clone());
        }
    }

    pub fn find_node(&self, query: &str) -> QueryResult {
        let collapsed_query = query.trim().replace(" ", "");

        if query == collapsed_query {
            log!(VERBOSE, "Chasing candidate for query {query}");
        } else {
            log!(
                VERBOSE,
                "Chasing candidate: query {query}, collapsed {collapsed_query}"
            );
        }

        let candidate = if let Some(exact_match) = self.nodes.get(query) {
            log!(VERBOSE, "Elected exact match {exact_match}");
            QueryResult {
                node: Some(exact_match.clone()),
                exact: true,
                redirect: false,
            }
        } else if let Some(lower_key) =
            self.lowercase_keymap.get(&collapsed_query.to_lowercase())
        {
            log!(
                VERBOSE,
                "Elected non-exact match through lower key {lower_key}"
            );
            QueryResult {
                node: self.nodes.get(lower_key).cloned(),
                exact: false,
                redirect: false,
            }
        } else {
            log!(VERBOSE, "No candidate found");
            QueryResult::default()
        };

        if let Some(candidate_node) = &candidate.node
            && !candidate_node.redirect.is_empty()
        {
            log!(VERBOSE, "Recursing: candidate is a redirect");
            if let Some(recursive_match) =
                self.find_node(&candidate_node.redirect).node
            {
                QueryResult {
                    node: Some(recursive_match),
                    exact: false,
                    redirect: true,
                }
            } else {
                QueryResult {
                    node: None,
                    exact: false,
                    redirect: true,
                }
            }
        } else {
            log!(VERBOSE, "Returning candidate {candidate}");
            candidate
        }
    }

    pub fn get_root(&self) -> Option<Node> {
        self.nodes.get(&self.root_node).cloned()
    }

    pub fn parse_config(&mut self) {
        self.meta.config.footer_text =
            content::parse(&self.meta.config.footer_text, self);
        self.meta.config.about_text =
            content::parse(&self.meta.config.about_text, self);
    }
}

pub enum Format {
    TOML,
    JSON,
    Unsupported,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SerialErrorCause {
    UnsupportedFormat,
    MalformedInput,
}

impl std::fmt::Display for SerialErrorCause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let text = match self {
            SerialErrorCause::MalformedInput => "Malformed Input",
            SerialErrorCause::UnsupportedFormat => "Unsupported Format",
        };
        write!(f, "{text}")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerialError {
    pub cause: SerialErrorCause,
    pub message: String,
}

impl From<SerialError> for String {
    fn from(error: SerialError) -> String {
        format!("{}: {}", error.cause, error.message)
    }
}

impl From<&str> for Format {
    fn from(s: &str) -> Format {
        if s.to_lowercase() == "toml" {
            Format::TOML
        } else if s.to_lowercase() == "json" {
            Format::JSON
        } else {
            Format::Unsupported
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Format::TOML => write!(f, "TOML"),
            Format::JSON => write!(f, "JSON"),
            Format::Unsupported => write!(f, "Unsupported format"),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct QueryResult {
    pub node: Option<Node>,
    pub redirect: bool,
    pub exact: bool,
}

impl std::fmt::Display for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let meta = if self.redirect && self.exact {
            "[exact redirect] "
        } else if !self.redirect && self.exact {
            "[exact] "
        } else if self.redirect && !self.exact {
            "[redirect] "
        } else {
            ""
        };
        let node = if let Some(n) = &self.node {
            n.id.clone()
        } else {
            String::from("No Match")
        };
        write!(f, "QueryResult: {meta}{node}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let graph = Graph::with_message("ISryQFd9peG6eYz9CFRQFWeD1GnPo0oj");
        assert!(graph.nodes.is_empty());
        assert!(graph.incoming.is_empty());
        assert_eq!(
            graph.meta.messages.first().unwrap(),
            "ISryQFd9peG6eYz9CFRQFWeD1GnPo0oj"
        );
    }

    #[test]
    fn good_json() {
        let json = r#"
        {
            "nodes": {
                "JSON": {
                    "text": "",
                    "title": "JSON",
                    "links": [],
                    "id": "JSON",
                    "hidden": false,
                    "connections": {}
                }
            },
            "root_node": "JSON"
        }
        "#;

        let deserialize_result = Graph::from_serial(json, &Format::JSON);
        println!("{deserialize_result:?}");
        assert!(deserialize_result.is_ok());
    }

    #[test]
    fn bad_json() {
        assert!(Graph::from_serial(":::", &Format::JSON).is_err_and(|e| {
            e.message.contains("expected value at line 1 column 1")
        },));
    }

    #[test]
    fn with_message() {
        let payload = "QmMxohuLe9DZCOzcaxH2wzZGqOot1In6";
        let graph = Graph::with_message(payload);
        assert_eq!(payload, graph.meta.messages.first().unwrap());
    }

    #[test]
    fn malformed_without_message() {
        let graph = Graph::malformed(None);
        assert!(graph.meta.messages.is_empty());
    }

    #[test]
    fn malformed_with_message() {
        let payload = "s8LuGwRQA4GdNGvAlaIUrryZYBGkY5Ev";
        let graph = Graph::malformed(Some(payload));
        assert_eq!(payload, graph.meta.messages.first().unwrap());
    }

    #[test]
    fn bad_deserial_input() {
        let result = Graph::from_serial("not toml", &Format::TOML);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().cause,
            SerialErrorCause::MalformedInput
        ));
    }

    #[test]
    fn bad_deserial_format() {
        let result = Graph::from_serial("not toml", &Format::Unsupported);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().cause,
            SerialErrorCause::UnsupportedFormat
        ));
    }

    #[test]
    fn bad_serial_format() {
        let result = Graph::load().to_serial(&Format::Unsupported);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().cause,
            SerialErrorCause::UnsupportedFormat
        ));
    }

    #[test]
    fn empty_modulated_graph_is_empty() {
        let mut graph = Graph::from_serial("", &Format::TOML).unwrap();
        graph.modulate();

        assert!(graph.nodes.is_empty());
        assert!(graph.incoming.is_empty());
    }

    #[test]
    fn title_population_from_id() {
        let mut graph = Graph::from_serial(
            concat!("[nodes.TitlelessNode]\n", r#"text = "Some text""#,),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let node = graph.nodes.get("TitlelessNode");
        assert_eq!(node.unwrap().title, "TitlelessNode");
    }

    #[test]
    fn no_title_population_from_id_if_title_set() {
        let mut graph = Graph::from_serial(
            concat!(
                "[nodes.TitlefulNode]\n",
                r#"title = "A Title""#,
                "\n",
                r#"text = "Some text""#,
            ),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let node = graph.nodes.get("TitlefulNode");
        assert_eq!(node.unwrap().title, "A Title");
    }

    #[test]
    fn detached_edge_is_flagged() {
        let mut graph = Graph::from_serial(
            concat!(
                "[nodes.Node]\n",
                r#"text = "Some text here""#,
                "\n\n",
                "[nodes.Node.connections.Nowhere]\n",
            ),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let node = graph.nodes.get("Node").unwrap();
        let connection = node.connections.get("Nowhere").unwrap();
        assert!(connection.detached);
    }

    #[test]
    fn attached_edge_is_not_flagged() {
        let mut graph = Graph::from_serial(
            concat!(
                "[nodes.NodeOne]\n",
                r#"text = "Some text here""#,
                "\n\n",
                "[nodes.NodeOne.connections.NodeTwo]\n\n",
                "[nodes.NodeTwo]\n",
                r#"text = "Some other text here""#,
                "\n\n",
            ),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let node = graph.nodes.get("NodeOne").unwrap();
        let connection = node.connections.get("NodeTwo").unwrap();
        println!("{connection:#?}");
        assert!(!connection.detached);
    }

    #[test]
    fn to_and_from_population() {
        let mut graph = Graph::from_serial(
            concat!(
                "[nodes.n01]\n",
                "[nodes.n01.connections.n02]\n\n",
                "[nodes.n02]\n",
                "[nodes.n02.connections.n03]\n\n",
            ),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let n01 = graph.nodes.get("n01").unwrap();
        let n02 = graph.nodes.get("n02").unwrap();
        let n01_to_n02 = n01.connections.get("n02").unwrap();
        let n02_to_n03 = n02.connections.get("n03").unwrap();

        assert_eq!(n01_to_n02.from, "n01");
        assert_eq!(n01_to_n02.to, "n02");
        assert!(!n01_to_n02.detached);
        assert_eq!(n02_to_n03.from, "n02");
        assert_eq!(n02_to_n03.to, "n03");
        assert!(n02_to_n03.detached);
    }

    #[test]
    fn links_become_connections() {
        let mut graph = Graph::from_serial(
            concat!(
                "[nodes.n01]\n",
                r#"links = [ "n02", "n03", "n04" ]"#,
                "\n\n",
                "[nodes.n02]\n",
                "[nodes.n04]\n",
                r#"links = [ "n01", "n03" ]"#,
                "\n\n",
            ),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let n01 = graph.nodes.get("n01").unwrap();
        let n02 = graph.nodes.get("n02").unwrap();
        let n04 = graph.nodes.get("n04").unwrap();

        let n01_to_n02 = n01.connections.get("n02").unwrap();
        let n01_to_n03 = n01.connections.get("n03").unwrap();
        let n01_to_n04 = n01.connections.get("n04").unwrap();

        let n04_to_n01 = n04.connections.get("n01").unwrap();
        let n04_to_n03 = n04.connections.get("n03").unwrap();

        assert_eq!(n01_to_n02.from, "n01");
        assert_eq!(n01_to_n02.to, "n02");
        assert!(!n01_to_n02.detached);

        assert_eq!(n01_to_n03.from, "n01");
        assert_eq!(n01_to_n03.to, "n03");
        assert!(n01_to_n03.detached);

        assert_eq!(n01_to_n04.from, "n01");
        assert_eq!(n01_to_n04.to, "n04");
        assert!(!n01_to_n04.detached);

        assert!(n02.connections.is_empty());

        assert_eq!(n04_to_n01.from, "n04");
        assert_eq!(n04_to_n01.to, "n01");
        assert!(!n04_to_n01.detached);

        assert_eq!(n04_to_n03.from, "n04");
        assert_eq!(n04_to_n03.to, "n03");
        assert!(n04_to_n03.detached);
    }

    #[test]
    fn detached_count_increments() {
        let mut graph = Graph::from_serial(
            concat!(
                "[nodes.n01]\n",
                r#"links = [ "n02", "n03", "n04", "n05", "n06", "n10" ]"#,
                "\n\n",
                "[nodes.n02]\n",
                "[nodes.n04]\n",
                r#"links = [ "n01", "n02", "n03", "n06", "n11", "n15" ]"#,
                "\n\n"
            ),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        assert_eq!(graph.stats.detached_total, 8);
    }

    #[test]
    fn populated_summary() {
        let text = "vh18qEUN22X2SxLj6lpOOzMBB4N6S0UG";
        let mut graph = Graph::from_serial(
            &format!(
                "[nodes.n01]\n\
            text = \"{text}\"\n\
            "
            ),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        assert!(graph.nodes.get("n01").unwrap().summary.contains(text));
        assert!(graph.nodes.get("n01").unwrap().text.contains(text));
    }

    #[test]
    fn supplied_summary() {
        let text = "vh18qEUN22X2SxLj6lpOOzMBB4N6S0UG";
        let summary = "W5dhPgNs7S1Zsq6uPK47MAw8xXyNxwep";
        let mut graph = Graph::from_serial(
            &format!(
                "[nodes.n01]\n\
            summary = \"{summary}\"\n\
            text = \"{text}\"\n\
            ",
            ),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        assert_eq!(graph.nodes.get("n01").unwrap().summary, summary);
        assert!(graph.nodes.get("n01").unwrap().text.contains(text));
    }

    #[test]
    fn summary_from_first_sentence() {
        let first_sentence = "zTWFX0a8 tYTO2g.";
        let text = format!(
            "{first_sentence} QoGa PtDsJ vh18qE U N22 X2S. MBB4N6S0UG\n\n\
            6FokUX o OCEc LzZFfR1nkqa hWIF LdrtD3G. PDQwv Ba2PnZ yEBVpqQdt\n\n\
            Py6aoPK FV7iU UdrYB vD UeMvvg u 5kbt 9ZW9x7MR"
        );
        let mut graph = Graph::from_serial(
            &format!("[nodes.n01]\ntext = \"\"\"{text}\"\"\"\n"),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        assert_eq!(graph.nodes.get("n01").unwrap().summary, first_sentence);
    }

    #[test]
    fn summary_from_first_paragraph() {
        let first_paragraph = "EGR6IS fTQsRp rv7 g jvItnYU 2HNciS MID\n\
            iz3vx vXxa vW4JI6l E5itd6qm2Yx gFw 1D Nq0805 bXHe3h iqABe ilnHKl\n\
            F4AHvMLto cz3C Z279r9 jtIbBnY JqjwZPQdepf cdv6";
        let text = format!(
            "{first_paragraph}\n\nn0D CvHcIU7R oPcZy V1Iy9PgXO gOw lfeDy\n\n\
            jrOSq 0uVtJLd Idx08Bpy BBj 4PVS R9lt RqjTs s AURUx93 Xu9WiI0rP.\n"
        );
        let mut graph = Graph::from_serial(
            format!("[nodes.n01]\ntext = \"\"\"{text}\"\"\"\n").as_str(),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        assert_eq!(graph.nodes.get("n01").unwrap().summary, first_paragraph);
    }

    #[test]
    fn summary_from_first_300_chars() {
        let first_300 = concat!(
            "Primis, quod, cum in rerum natura duo ",
            "quaerenda sint, unum, quae materia sit, ex qua quaeque res ",
            "efficiatur, alterum, quae naturales essent nec tamen id, cuius ",
            "causa haec finxerat, assecutus est: Nam si omnes veri erunt, ut ",
            "Epicuri ratio docet, tum denique poterit aliquid cognosci et ",
            "percipi? Quos q"
        );
        let tail = concat!(
            "uam autem et praeterita grate meminit et ",
            "praesentibus ita potitur, ut animadvertat quanta sint ea ",
            "quamque iucunda, neque pendet ex futuris, sed expectat illa, ",
            "fruitur praesentibus ab iisque vitii"
        );
        let text = format!("{first_300}{tail}");
        let summary = format!("{first_300}…");

        let mut graph = Graph::from_serial(
            format!("[nodes.n01]\ntext = \"\"\"{text}\"\"\"\n").as_str(),
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        assert_eq!(graph.nodes.get("n01").unwrap().summary, summary);
    }

    #[test]
    fn anchors_become_connections() {
        let mut graph = Graph::from_serial(
            "\
            [nodes.n1]
            text = 'an anchor to |n2|, the existing node'

            [nodes.n2]
            text = 'an anchor to |n0|, the nonexistent node'

        ",
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let n1_to_n2 = graph.nodes.get("n1").unwrap().connections.get("n2");

        let n2_to_n0 = graph.nodes.get("n2").unwrap().connections.get("n0");

        println!("{n1_to_n2:#?}");
        println!("{n2_to_n0:#?}");

        assert!(!n1_to_n2.unwrap().detached);
        assert!(n2_to_n0.unwrap().detached);
    }

    #[test]
    fn detached_anchors_increase_counts() {
        let mut graph = Graph::from_serial(
            "\
            [nodes.n1]\n\
            text = 'this |anchor| is detached, as is |this one|.'\n\
            ",
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        assert_eq!(graph.stats.detached_total, 2);
        assert_eq!(*graph.stats.detached.get("anchor").unwrap(), 1);
        assert_eq!(*graph.stats.detached.get("this one").unwrap(), 1);
    }

    #[test]
    fn repeated_detached_anchors_increase_counts() {
        let mut graph = Graph::from_serial(
            "\
            [nodes.n1]\n\
            text = 'this |anchor| is detached, as appears twice: |anchor|.'\n\
            ",
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        assert_eq!(graph.stats.detached_total, 2);
        assert_eq!(*graph.stats.detached.get("anchor").unwrap(), 2);
    }

    #[test]
    fn find_exact() {
        let mut graph = Graph::from_serial(
            "\
            [nodes.n1]\n\
            ",
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let query_result = graph.find_node("n1");
        assert_eq!(query_result.node.unwrap().id, "n1");
        assert!(query_result.exact);
        assert!(!query_result.redirect);
    }

    #[test]
    fn find_inexact() {
        let mut graph = Graph::from_serial(
            "\
            [nodes.n1]\n\
            ",
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let query_result = graph.find_node("n 1");
        println!("{query_result}");

        assert_eq!(query_result.node.unwrap().id, "n1");
        assert!(!query_result.exact);
        assert!(!query_result.redirect);
    }

    #[test]
    fn find_inexact_to_redirect() {
        let mut graph = Graph::from_serial(
            "\
            [nodes.n1]\n\
            redirect = 'n3'
            \n\
            [nodes.n3]
            ",
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let query_result = graph.find_node("n 1");
        println!("{query_result}");

        assert_eq!(query_result.node.unwrap().id, "n3");
        assert!(!query_result.exact);
        assert!(query_result.redirect);
    }

    #[test]
    fn double_recursion_to_redirect() {
        let mut graph = Graph::from_serial(
            "\
            [nodes.n1]\n\
            redirect = 'n2'
            \n\
            [nodes.n2]\n\
            redirect = 'n 3'
            \n\
            [nodes.n3]
            ",
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let query_result = graph.find_node("n 1");
        println!("{query_result}");

        assert_eq!(query_result.node.unwrap().id, "n3");
        assert!(!query_result.exact);
        assert!(query_result.redirect);
    }

    #[test]
    fn find_redirect_to_inexisting() {
        let mut graph = Graph::from_serial(
            "\
            [nodes.n1]\n\
            redirect = 'n0'\n\
            ",
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let query_result = graph.find_node("n1");
        println!("{query_result}");

        assert!(query_result.node.is_none());
        assert!(query_result.redirect);
        assert!(!query_result.exact);
    }

    #[test]
    fn find_redirect_to_existing() {
        let mut graph = Graph::from_serial(
            "\
            [nodes.n1]\n\
            redirect = 'n2'\n\
            \n\
            [nodes.n2]
            ",
            &Format::TOML,
        )
        .unwrap();
        graph.modulate();

        let query_result = graph.find_node("n1");
        assert_eq!(query_result.node.unwrap().id, "n2");
        assert!(query_result.redirect);
        assert!(!query_result.exact);
    }

    #[test]
    fn serial_error_display() {
        let bad_input = SerialErrorCause::MalformedInput;
        let bad_format = SerialErrorCause::UnsupportedFormat;

        assert_eq!(format!("{bad_input}"), "Malformed Input");
        assert_eq!(format!("{bad_format}"), "Unsupported Format");
    }

    #[test]
    fn string_from_serial_error() {
        let bad_input_error_message = "denSehpfhCjr05gUd7TgYLb8veJHAMZW";
        let bad_input_cause = SerialErrorCause::MalformedInput;
        let bad_input = SerialError {
            cause: bad_input_cause.clone(),
            message: bad_input_error_message.to_string(),
        };

        let bad_format_error_message = "4brcCkWOgLHBvhLk2OcgTOKQgpKrc1bB";
        let bad_format_cause = SerialErrorCause::UnsupportedFormat;
        let bad_format = SerialError {
            cause: bad_format_cause.clone(),
            message: bad_format_error_message.to_string(),
        };

        let s_bad_input = String::from(bad_input.clone());
        let s_bad_format = String::from(bad_format.clone());

        assert!(s_bad_input.contains(bad_input_error_message));
        assert!(s_bad_input.contains(bad_input.message.as_str()));
        assert!(s_bad_input.contains(format!("{bad_input_cause}").as_str()));

        assert!(s_bad_format.contains(bad_format_error_message));
        assert!(s_bad_format.contains(bad_format.message.as_str()));
        assert!(s_bad_format.contains(format!("{bad_format_cause}").as_str()));
    }

    #[test]
    fn format_from_str() {
        let uppercase_toml = Format::from("TOML");
        let lowercase_toml = Format::from("toml");
        let mixed_case_toml = Format::from("tOmL");

        assert!(matches!(uppercase_toml, Format::TOML));
        assert!(matches!(lowercase_toml, Format::TOML));
        assert!(matches!(mixed_case_toml, Format::TOML));

        let uppercase_json = Format::from("JSON");
        let lowercase_json = Format::from("json");
        let mixed_case_json = Format::from("JsoN");

        assert!(matches!(uppercase_json, Format::JSON));
        assert!(matches!(lowercase_json, Format::JSON));
        assert!(matches!(mixed_case_json, Format::JSON));

        let unsupported = [
            Format::from("j son"),
            Format::from(""),
            Format::from("strawberry"),
            Format::from(" "),
            Format::from("\n"),
        ];

        assert!(unsupported.iter().all(|f| matches!(f, Format::Unsupported)));
    }

    #[test]
    fn format_display() {
        let toml = format!("{}", Format::TOML);
        let json = format!("{}", Format::JSON);
        let unsupported = format!("{}", Format::Unsupported);

        assert_eq!(toml, "TOML");
        assert_eq!(json, "JSON");
        assert_eq!(unsupported, "Unsupported format");
    }

    #[test]
    fn query_result_display() {
        let mut node = Node::default();
        let node_id = "nv00qmO6PDrqJheUHOONlCVpuceefS30";
        node.id = String::from(node_id);

        let none_exact = QueryResult {
            node: None,
            exact: true,
            redirect: false,
        };

        assert!(!format!("{none_exact}").contains(node_id));
        assert!(format!("{none_exact}").contains("No Match"));
        assert!(format!("{none_exact}").contains("exact"));
        assert!(!format!("{none_exact}").contains("redirect"));

        let some_exact = QueryResult {
            node: Some(node.clone()),
            exact: true,
            redirect: false,
        };

        assert!(format!("{some_exact}").contains(node_id));
        assert!(!format!("{some_exact}").contains("No Match"));
        assert!(format!("{some_exact}").contains("exact"));
        assert!(!format!("{some_exact}").contains("redirect"));

        let none_redirect = QueryResult {
            node: None,
            exact: false,
            redirect: true,
        };

        assert!(!format!("{none_redirect}").contains(node_id));
        assert!(format!("{none_redirect}").contains("No Match"));
        assert!(format!("{none_redirect}").contains("redirect"));
        assert!(!format!("{none_redirect}").contains("exact"));

        let some_redirect = QueryResult {
            node: Some(node.clone()),
            exact: false,
            redirect: true,
        };

        assert!(format!("{some_redirect}").contains(node_id));
        assert!(!format!("{some_redirect}").contains("No Match"));
        assert!(format!("{some_redirect}").contains("redirect"));
        assert!(!format!("{some_redirect}").contains("exact"));

        let some_exact_redirect = QueryResult {
            node: Some(node.clone()),
            exact: true,
            redirect: true,
        };

        assert!(format!("{some_exact_redirect}").contains(node_id));
        assert!(!format!("{some_exact_redirect}").contains("No Match"));
        assert!(format!("{some_exact_redirect}").contains("redirect"));
        assert!(format!("{some_exact_redirect}").contains("exact"));

        let none_exact_redirect = QueryResult {
            node: None,
            exact: true,
            redirect: true,
        };

        assert!(!format!("{none_exact_redirect}").contains(node_id));
        assert!(format!("{none_exact_redirect}").contains("No Match"));
        assert!(format!("{none_exact_redirect}").contains("redirect"));
        assert!(format!("{none_exact_redirect}").contains("exact"));

        let none = QueryResult {
            node: None,
            exact: false,
            redirect: false,
        };

        assert!(!format!("{none}").contains(node_id));
        assert!(format!("{none}").contains("No Match"));
        assert!(!format!("{none}").contains("redirect"));
        assert!(!format!("{none}").contains("exact"));

        let some = QueryResult {
            node: Some(node.clone()),
            exact: false,
            redirect: false,
        };

        assert!(format!("{some}").contains(node_id));
        assert!(!format!("{some}").contains("No Match"));
        assert!(!format!("{some}").contains("redirect"));
        assert!(!format!("{some}").contains("exact"));
    }
}

#[cfg(test)]
mod serial_tests {
    use super::*;

    #[test]
    fn bad_graph_path() {
        let original_working_directory = std::env::current_dir().unwrap();

        assert!(
            std::env::set_current_dir(std::path::Path::new(
                "tests/mocks/no_graph"
            ))
            .is_ok()
        );

        let graph = Graph::load();
        let message = graph.meta.messages.first().unwrap();
        assert!(message.contains("Failed reading file at"));

        assert!(std::env::set_current_dir(original_working_directory).is_ok());
    }
}
