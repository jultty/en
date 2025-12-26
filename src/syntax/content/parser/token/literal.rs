use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

#[derive(Debug)]
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
        self.text.clone()
    }
}
