use crate::{
    prelude::*,
    syntax::content::{Parseable, Lexeme},
};
use std::fmt::Display;

pub struct Header {
    open: Option<bool>,
    level: Level,
}

impl Header {
    pub fn new(level: Level, open: bool) -> Header {
        Header {
            level,
            open: Some(open),
        }
    }

    pub fn from_u8(level: u8, open: bool) -> Header {
        Header {
            level: Level::from_u8(level),
            open: Some(open),
        }
    }

    pub fn get_level(&self) -> u8 {
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
        Header::new(lexeme.text().len().into(), true)
    }

    fn render(&self) -> String {
        if let Some(open) = self.open {
            if open {
                format!("<h{}>", &self.level)
            } else {
                format!("</h{}>", &self.level)
            }
        } else {
            panic!("Attempt to render a header tag while open state is unknown")
        }
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(open) = self.open {
            if open {
                write!(f, "Level {} Open Header", self.level)
            } else {
                write!(f, "Level {} Closed Header", self.level)
            }
        } else {
            write!(f, "Level {} Header (Unknown open state)", self.level)
        }
    }
}

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
