use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::syntax::content::parsers::{compound::elements::literal::Literal, line::elements::{paragraph::Paragraph, span::Span}};

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Graph {
    pub nodes: HashMap<String, Node>,
    pub root_node: String,
    #[serde(skip_deserializing)]
    pub incoming: HashMap<String, Vec<Edge>>,
    #[serde(default)]
    pub meta: Meta,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Node {
    pub text: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub links: Vec<String>,
    #[serde(default)]
    pub id: String,

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

#[expect(clippy::struct_excessive_bools)]
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Config {
    #[serde(default)]
    pub site_title: String,
    #[serde(default)]
    pub site_description: String,
    #[serde(default = "mktrue")]
    pub footer: bool,
    #[serde(default = "mktrue")]
    pub footer_credits: bool,
    #[serde(default = "mktrue")]
    pub footer_date: bool,
    #[serde(default)]
    pub footer_text: String,
    #[serde(default = "mktrue")]
    pub about: bool,
    #[serde(default)]
    pub about_text: String,
    #[serde(default = "mktrue")]
    pub tree: bool,
    #[serde(default = "mktrue")]
    pub raw: bool,
    #[serde(default = "mktrue")]
    pub raw_toml: bool,
    #[serde(default = "mktrue")]
    pub raw_json: bool,
    #[serde(default = "mktrue")]
    pub index_search: bool,
    #[serde(default = "mktrue")]
    pub index_node_list: bool,
    #[serde(default = "mk8")]
    pub index_node_count: u16,
    #[serde(default = "mktrue")]
    pub index_root_node: bool,
    #[serde(default = "mktrue")]
    pub tree_node_text: bool,
}

// See: https://github.com/serde-rs/serde/issues/368
fn mktrue() -> bool {
    true
}
fn mk8() -> u16 {
    8
}

impl Graph {
    pub fn new(message: Option<String>) -> Graph {
        Graph {
            nodes: HashMap::new(),
            root_node: "VoidNode".to_string(),
            incoming: HashMap::new(),
            meta: Meta {
                config: Config {
                    site_title: String::new(),
                    site_description: String::new(),
                    footer: true,
                    footer_credits: true,
                    footer_date: true,
                    footer_text: String::new(),
                    about: true,
                    about_text: String::new(),
                    tree: true,
                    raw: true,
                    raw_toml: true,
                    raw_json: true,
                    index_search: true,
                    index_node_list: true,
                    index_node_count: 8,
                    index_root_node: true,
                    tree_node_text: true,
                },
                version: (0, 1, 0),
                messages: message.map_or(vec![], |m| vec![m]),
            },
        }
    }

    pub fn get_root(&self) -> Option<Node> {
        self.nodes.get(&self.root_node).cloned()
    }
}

impl Node {
    pub fn new(message: Option<String>) -> Node {
        Node {
            id: "VoidNode".to_string(),
            title: "Pure Void".to_string(),
            text: match message {
                Some(s) => s,
                None => "Node is empty, missing or wasn't found.".to_string(),
            },
            connections: None,
            links: vec![],
        }
    }
}

impl Config {
    #[must_use]
    pub fn parse_text(self) -> Config {

        Config {
            footer_text: crate::syntax::content::parse::<Span, Literal>(
                &self.footer_text,
            ),
            about_text: crate::syntax::content::parse::<Paragraph, Literal>(
                &self.about_text,
            ),
            ..self
        }
    }
}
