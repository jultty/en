use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct Meta {
    pub config: Config,
    #[serde(default = "mkversion")]
    pub version: (u8, u8, u8),
    #[serde(default)]
    pub messages: Vec<String>,
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
    pub tree_node_summary: bool,
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
            tree_node_summary: false,
        }
    }
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
fn mkversion() -> (u8, u8, u8) {
    (0, 0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::serial::populate_graph;

    #[test]
    fn empty_footer_text() {
        let mut graph = populate_graph();

        graph.meta.config = Config {
            footer_text: String::default(),
            ..graph.meta.config
        };

        graph.parse();

        println!("{:?}", graph.meta.config.footer_text);
        assert!(graph.meta.config.footer_text.is_empty());
    }

    #[test]
    fn config_footer_text() {
        let payload = "0kqBrdS8NPrU4xVxh2xW0hUzAw926JCQ";
        let mut graph = populate_graph();

        graph.meta.config = Config {
            footer_text: format!("`{payload}`"),
            ..graph.meta.config
        };

        graph.parse();

        assert!(
            graph
                .meta
                .config
                .footer_text
                .matches(format!("<code>{payload}</code>").as_str())
                .count()
                == 1
        );
    }

    #[test]
    fn config_about_text() {
        let payload = "ZqPFl84JlzSS0QUo61RwTUPONIE78Lmw";
        let mut graph = populate_graph();

        graph.meta.config = Config {
            about_text: format!("`{payload}`"),
            ..graph.meta.config
        };

        graph.parse();

        assert!(
            graph
                .meta
                .config
                .about_text
                .matches(format!("<code>{payload}</code>").as_str())
                .count()
                == 1
        );
    }
}
