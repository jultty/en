use std::fmt::Display;
use crate::syntax::content::{Parseable, Lexeme};

pub(in crate::syntax::content) struct Paragraph {
    text: String,
}

impl Parseable for Paragraph {
    fn probe(lexeme: &Lexeme) -> bool {
        !lexeme.raw.trim().is_empty()
    }

    fn lex(lexeme: &Lexeme) -> Self {
        Self {
            text: lexeme.raw.trim().to_owned(),
        }
    }

    fn render(&self) -> String {
        format!("<p>{}</p>", &self.text)
    }
}

impl Display for Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Paragraph: <{}>", &self.text)
    }
}
