use crate::{
    syntax::content::{Parseable, parser::Lexeme},
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

    fn flatten(&self) -> String {
        String::from('\n')
    }
}

impl std::fmt::Display for LineBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "LineBreak")
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::content::parser::Token;

    use super::*;

    #[test]
    fn token_display() {
        let linebreak = LineBreak::default();
        assert_eq!(format!("{}", Token::LineBreak(linebreak)), "Tk:LineBreak");
    }
}
