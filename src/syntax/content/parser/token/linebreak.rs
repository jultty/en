use crate::{
    syntax::content::{Parseable, parser::lexeme::Lexeme},
};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct LineBreak {}

impl Parseable for LineBreak {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.text() == "\n" && !lexeme.last()
    }

    fn lex(_lexeme: &Lexeme) -> LineBreak {
        LineBreak {}
    }

    fn render(&self) -> String {
        "\n".to_owned()
    }
}
