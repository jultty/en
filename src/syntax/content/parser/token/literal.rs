use std::fmt::Display;
use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

pub struct Literal {
    text: String,
}

impl Parseable for Literal {
    fn probe(_lexeme: &Lexeme) -> bool {
        true
    }

    fn lex(lexeme: &Lexeme) -> Literal {
        Literal {
            text: lexeme.to_raw(),
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
