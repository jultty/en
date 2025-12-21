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
            text: lexeme.text(),
        }
    }

    fn render(&self) -> String {
        let non_sticky = [" ", "\n"];
        if non_sticky.contains(&self.text.as_str()) {
            self.text.clone()
        } else {
            format!("{} ", self.text.clone())
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Literal: <{}>", &self.text)
    }
}
