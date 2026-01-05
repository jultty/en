use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Literal {
    text: String,
}

impl Parseable for Literal {
    fn probe(lexeme: &Lexeme) -> bool {
        !(lexeme.last() && lexeme.is_whitespace())
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
