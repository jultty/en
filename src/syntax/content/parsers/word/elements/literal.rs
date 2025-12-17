use std::fmt::Display;
use crate::syntax::content::{Parseable, lexeme::Lexeme};

pub struct Literal {
    text: String,
}

impl Parseable for Literal {
    fn probe(lexeme: &Lexeme) -> bool {
        !lexeme.to_raw().is_empty()
    }

    fn lex(lexeme: &Lexeme) -> Literal {
        Literal {
            text: lexeme.to_raw().trim().to_owned(),
        }
    }

    fn render(&self) -> String {
        self.text.clone()
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Literal: <{}>", &self.text)
    }
}
