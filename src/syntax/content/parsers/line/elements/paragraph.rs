use std::fmt::Display;
use crate::syntax::content::{Parseable, lexeme::Lexeme};

pub struct Paragraph {
    text: String,
}

impl Parseable for Paragraph {
    fn probe(lexeme: &Lexeme) -> bool {
        !lexeme.to_raw().trim().is_empty()
    }

    fn lex(lexeme: &Lexeme) -> Paragraph {
        Paragraph {
            text: lexeme.to_raw().trim().to_owned(),
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
