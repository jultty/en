use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::syntax::content;

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Graph {
    pub nodes: HashMap<String, Node>,
    pub root_node: String,
    #[serde(skip_deserializing)]
    pub incoming: HashMap<String, Vec<Edge>>,
    #[serde(skip_deserializing)]
    pub lowercase_keymap: HashMap<String, String>,
    #[serde(default)]
    pub meta: Meta,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Node {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub links: Vec<String>,
    #[serde(default)]
    pub redirect: String,
    #[serde(default)]
    pub hidden: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<Edge>>,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Edge {
    pub to: String,
    #[serde(default)]
    pub anchor: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub detached: bool,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Meta {
    pub config: Config,
    #[serde(default = "mkversion")]
    pub version: (u8, u8, u8),
    #[serde(default)]
    pub messages: Vec<String>,
}

// See: https://github.com/serde-rs/serde/issues/368
fn mkversion() -> (u8, u8, u8) {
    (0, 0, 0)
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Config {
    #[serde(default)]
    _private: bool,
    #[serde(default = "mktrue")]
    pub about: bool,
    #[serde(default)]
    pub about_text: String,
    #[serde(default = "mkfalse")]
    pub ascii_dom_ids: bool,
    #[serde(default)]
    pub content_language: String,
    #[serde(default = "mkfalse")]
    error_poem: bool,
    #[serde(default = "mktrue")]
    pub footer: bool,
    #[serde(default = "mktrue")]
    pub footer_credits: bool,
    #[serde(default = "mktrue")]
    pub footer_date: bool,
    #[serde(default)]
    pub footer_text: String,
    #[serde(default = "mk8")]
    pub index_node_count: u16,
    #[serde(default = "mktrue")]
    pub index_node_list: bool,
    #[serde(default = "mktrue")]
    pub index_root_node: bool,
    #[serde(default = "mktrue")]
    pub index_search: bool,
    #[serde(default)]
    node_selector: bool,
    #[serde(default)]
    navbar_search: bool,
    #[serde(default = "mktrue")]
    pub raw: bool,
    #[serde(default = "mktrue")]
    pub raw_json: bool,
    #[serde(default = "mktrue")]
    pub raw_toml: bool,
    #[serde(default)]
    pub site_description: String,
    #[serde(default)]
    pub site_title: String,
    #[serde(default = "mktrue")]
    pub tree: bool,
    #[serde(default = "mkfalse")]
    pub tree_node_text: bool,
}

// See: https://github.com/serde-rs/serde/issues/368
fn mktrue() -> bool {
    true
}
fn mkfalse() -> bool {
    false
}
fn mk8() -> u16 {
    8
}

pub struct QueryResult {
    pub node: Option<Node>,
    pub redirect: bool,
}

impl Graph {
    pub fn new(message: Option<&str>) -> Graph {
        Graph {
            nodes: HashMap::default(),
            root_node: "VoidNode".to_string(),
            incoming: HashMap::default(),
            lowercase_keymap: HashMap::default(),
            meta: Meta {
                config: Config::default(),
                version: (0, 1, 0),
                messages: message.map_or(vec![], |m| vec![m.to_string()]),
            },
        }
    }

    pub fn find_node(&self, query: &str) -> QueryResult {
        let collapsed_query = query.trim().replace(" ", "");

        if let Some(exact_match) = self.nodes.get(query) {
            QueryResult {
                node: Some(exact_match.clone()),
                redirect: false,
            }
        } else if let Some(lower_key) =
            self.lowercase_keymap.get(&collapsed_query.to_lowercase())
        {
            QueryResult {
                node: self.nodes.get(lower_key).cloned(),
                redirect: true,
            }
        } else {
            QueryResult {
                node: None,
                redirect: false,
            }
        }
    }

    pub fn get_root(&self) -> Option<Node> {
        self.nodes.get(&self.root_node).cloned()
    }
}

impl Node {
    pub fn new(message: Option<String>) -> Node {
        Node {
            id: "404".to_string(),
            title: "Not Found".to_string(),
            text: match message {
                Some(s) => s,
                None => "Node not found.".to_string(),
            },
            connections: None,
            links: vec![],
            redirect: String::default(),
            hidden: false,
        }
    }
}

impl Config {
    #[must_use]
    pub fn parse_text(self) -> Config {
        Config {
            footer_text: content::parse(&self.footer_text, &self),
            about_text: content::parse(&self.about_text, &self),
            ..self
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            _private: true,
            about: true,
            about_text: String::default(),
            ascii_dom_ids: false,
            content_language: String::default(),
            error_poem: false,
            footer: true,
            footer_credits: true,
            footer_date: true,
            footer_text: String::default(),
            index_node_count: 8,
            index_node_list: true,
            index_root_node: true,
            index_search: true,
            node_selector: true,
            navbar_search: true,
            raw: true,
            raw_json: true,
            raw_toml: true,
            site_description: String::default(),
            site_title: String::default(),
            tree: true,
            tree_node_text: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::serial::populate_graph;

    use super::*;

    #[test]
    fn empty_graph() {
        let graph = Graph::new(Some("ISryQFd9peG6eYz9CFRQFWeD1GnPo0oj"));
        assert!(graph.nodes.is_empty());
        assert!(graph.incoming.is_empty());
        assert_eq!(
            graph.meta.messages.first().unwrap(),
            "ISryQFd9peG6eYz9CFRQFWeD1GnPo0oj"
        );
    }

    #[test]
    fn empty_node_message() {
        let node = Node::new(None);
        assert_eq!(node.text, "Node not found.");
    }

    #[test]
    fn empty_footer_text() {
        let default_graph = populate_graph();

        let config = Config {
            footer_text: String::default(),
            ..default_graph.meta.config
        };

        let parsed_config = config.parse_text();

        println!("{:?}", parsed_config.footer_text);
        assert!(parsed_config.footer_text.is_empty());
    }

    #[test]
    fn config_footer_text() {
        let payload = "0kqBrdS8NPrU4xVxh2xW0hUzAw926JCQ";
        let default_graph = populate_graph();

        let config = Config {
            footer_text: format!("`{payload}`"),
            ..default_graph.meta.config
        };

        let parsed_config = config.parse_text();

        assert!(
            parsed_config
                .footer_text
                .matches(format!("<code>{payload}</code>").as_str())
                .count()
                == 1
        );
    }

    #[test]
    fn config_about_text() {
        let payload = "ZqPFl84JlzSS0QUo61RwTUPONIE78Lmw";
        let default_graph = populate_graph();

        let config = Config {
            about_text: format!("`{payload}`"),
            ..default_graph.meta.config
        };

        let parsed_config = config.parse_text();

        assert!(
            parsed_config
                .about_text
                .matches(format!("<code>{payload}</code>").as_str())
                .count()
                == 1
        );
    }
}
