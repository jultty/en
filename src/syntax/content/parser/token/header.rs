use std::{
    collections::{HashMap, hash_map::Entry},
};

use crate::{
    prelude::*,
    types::Config,
    syntax::content::{Parseable, Lexeme},
};

use std::fmt::Display;

#[derive(Debug)]
pub struct Header {
    open: Option<bool>,
    level: Level,
    pub dom_id: Option<String>,
}

impl Header {
    pub fn new(level: Level, open: bool, dom_id: Option<&str>) -> Header {
        Header {
            open: Some(open),
            level,
            dom_id: dom_id.map(std::borrow::ToOwned::to_owned),
        }
    }

    pub fn make_id(
        config: &Config,
        next_lexeme: &Lexeme,
        ids: &mut HashMap<String, Vec<String>>,
    ) -> String {
        let base_id = if !config.ascii_dom_ids || next_lexeme.next.is_ascii() {
            next_lexeme.next.clone()
        } else {
            String::from("h")
        };

        match ids.entry(base_id.clone()) {
            Entry::Occupied(mut occupied) => {
                let ids_vec = occupied.get_mut();
                let suffix = ids_vec.len();
                let id_with_suffix = format!("{base_id}-{suffix}");
                ids_vec.push(id_with_suffix.clone());
                id_with_suffix
            },
            Entry::Vacant(vacant) => {
                vacant.insert(vec![base_id.clone()]);
                base_id
            },
        }
    }

    pub fn from_u8(level: u8, open: bool, dom_id: Option<&str>) -> Header {
        Header {
            level: Level::from_u8(level),
            open: Some(open),
            dom_id: dom_id.map(std::borrow::ToOwned::to_owned),
        }
    }

    pub fn level(&self) -> u8 {
        match self.level {
            Level::One => 1,
            Level::Two => 2,
            Level::Three => 3,
            Level::Four => 4,
            Level::Five => 5,
            Level::Six => 6,
        }
    }
}

impl Parseable for Header {
    fn probe(lexeme: &Lexeme) -> bool {
        if lexeme
            .split_chars()
            .into_iter()
            .filter(|e| *e != '#')
            .count()
            == 0
        {
            let level = lexeme.text().len();
            lexeme.clone().split_words().len() == 1 && level > 0 && level <= 6
        } else {
            false
        }
    }

    fn lex(lexeme: &Lexeme) -> Header {
        Header::new(
            lexeme.text().len().into(),
            true,
            Some(&lexeme.next.to_ascii_lowercase()),
        )
    }

    fn render(&self) -> String {
        if let Some(open) = self.open {
            if open && let Some(ref id) = self.dom_id {
                format!(r#"<h{} id="{}">"#, self.level, id)
            } else if open {
                format!("<h{}>", self.level)
            } else {
                format!("</h{}>", self.level)
            }
        } else {
            panic!("Attempt to render a header tag while open state is unknown")
        }
    }
}

#[derive(Debug)]
pub enum Level {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

impl Level {
    fn from_u8(u: u8) -> Level {
        if u <= 1 {
            Level::One
        } else if u == 2 {
            Level::Two
        } else if u == 3 {
            Level::Three
        } else if u == 4 {
            Level::Four
        } else if u == 5 {
            Level::Five
        } else {
            Level::Six
        }
    }
}

impl From<usize> for Level {
    fn from(z: usize) -> Level {
        let u8 = match u8::try_from(z) {
            Ok(u) => u,
            Err(e) => {
                log!("Truncating header level {z} to 6: {e:?}");
                6
            },
        };
        Level::from_u8(u8)
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Level::One => write!(f, "1"),
            Level::Two => write!(f, "2"),
            Level::Three => write!(f, "3"),
            Level::Four => write!(f, "4"),
            Level::Five => write!(f, "5"),
            Level::Six => write!(f, "6"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_id() {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        let id = Header::make_id(
            &Config::default(),
            &Lexeme::new("##", "Title"),
            &mut map,
        );
        assert_eq!(id, "Title");
    }

    #[test]
    fn ascii_ids_set() {
        let config = Config {
            ascii_dom_ids: true,
            ..Config::default()
        };

        let id = Header::make_id(
            &config,
            &Lexeme::new("##", "駄目！"),
            &mut HashMap::new(),
        );
        assert_eq!(id, "h");
    }

    #[test]
    fn ascii_ids_unset() {
        let config = Config {
            ascii_dom_ids: false,
            ..Config::default()
        };

        let id = Header::make_id(
            &config,
            &Lexeme::new("##", "駄目！"),
            &mut HashMap::new(),
        );
        assert_eq!(id, "駄目！");
    }

    #[test]
    fn id_deduplication() {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        let config = Config::default();
        let id =
            Header::make_id(&config, &Lexeme::new("##", "UVrcCUjoQ"), &mut map);
        assert_eq!(id, "UVrcCUjoQ");

        let double =
            Header::make_id(&config, &Lexeme::new("##", "UVrcCUjoQ"), &mut map);
        assert_eq!(double, "UVrcCUjoQ-1");

        let double2 =
            Header::make_id(&config, &Lexeme::new("##", "UVrcCUjoQ"), &mut map);
        assert_eq!(double2, "UVrcCUjoQ-2");
    }

    #[test]
    fn get_level() {
        for l in 1..=6 {
            let header = Header::from_u8(l, true, None);
            assert_eq!(header.level(), l);
        }
    }

    #[test]
    fn no_id_render() {
        let open_header = Header::from_u8(2, true, None);
        let closed_header = Header::from_u8(2, false, None);
        assert_eq!(open_header.render(), "<h2>");
        assert_eq!(closed_header.render(), "</h2>");
    }

    #[test]
    #[should_panic(
        expected = "Attempt to render a header tag while open state is unknown"
    )]
    fn unknown_open_state_render() {
        let header = Header {
            level: Level::Two,
            open: None,
            dom_id: None,
        };

        header.render();
    }
}
