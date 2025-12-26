use std::fmt::Display;
use crate::{
    syntax::content::{Parseable, parser::lexeme::Lexeme},
};

#[derive(Debug)]
pub struct LineBreak {}

impl Parseable for LineBreak {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.text() == "\n"
    }

    fn lex(_lexeme: &Lexeme) -> LineBreak {
        LineBreak {}
    }

    fn render(&self) -> String {
        "\n".to_owned()
    }
}

impl Display for LineBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Line Break")
    }
}
