use crate::{
    dev::log,
    syntax::content::{Parseable, Lexeme},
};
use std::fmt::Display;

enum Level {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
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

pub struct Header {
    level: Level,
    text: String,
}

impl Header {
    fn new(level: usize, text: &str) -> Header {
        Header {
            level: match level {
                1 => Level::One,
                2 => Level::Two,
                3 => Level::Three,
                4 => Level::Four,
                5 => Level::Five,
                6 => Level::Six,
                _ => {
                    panic!("Attempted to construct a header with invalid level")
                },
            },
            text: text.to_owned(),
        }
    }
}

impl Parseable for Header {
    fn probe(lexeme: &Lexeme) -> bool {
        let first = lexeme.clone().first().unwrap_or_default();
        !first.trim().is_empty()
            && first.replace("#", "").is_empty()
            && first.len() <= 6
    }

    fn lex(lexeme: &Lexeme) -> Header {
        let first = lexeme.clone().first().unwrap_or_else(|| unreachable!());
        let header_level = &first.len();
        log(&Header::lex, &format!("Header level is {header_level}"));

        let header_text = lexeme.to_raw().replace(&first, "");

        Header::new(*header_level, &header_text)
    }

    fn render(&self) -> String {
        format!("<h{}>{}</h{0}>", &self.level, self.text)
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Level {} Header: <{}>", &self.level, self.text)
    }
}
