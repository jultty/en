use crate::syntax::content::{Parseable, Lexeme};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Item {
    pub text: String,
    pub depth: Option<u8>,
}

impl Parseable for Item {
    fn probe(_: &Lexeme) -> bool {
        false
    }

    fn lex(_: &Lexeme) -> Item {
        panic!("Attempt to lex an item directly from a lexeme")
    }

    fn render(&self) -> String {
        panic!("Items should only be rendered by a list's render method")
    }
}

impl Item {
    pub fn new(text: &str, depth: Option<u8>) -> Item {
        Item {
            text: String::from(text),
            depth,
        }
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Item [{}] {}",
            if let Some(depth) = self.depth {
                format!("D{depth}")
            } else {
                "<unknown>".to_string()
            },
            self.text,
        )
    }
}
