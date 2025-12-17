use std::fmt::Display;
use crate::syntax::content::{Parseable, lexeme::Lexeme};

pub struct Span {
    text: String,
}

impl Parseable for Span {
    fn probe(lexeme: &Lexeme) -> bool {
        !lexeme.to_raw().trim().is_empty()
    }

    fn lex(lexeme: &Lexeme) -> Span {
        Span {
            text: lexeme.to_raw().trim().to_owned(),
        }
    }

    fn render(&self) -> String {
        format!("<span>{}</span>", &self.text)
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Span: <{}>", &self.text)
    }
}
