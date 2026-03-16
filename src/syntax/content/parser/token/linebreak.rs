use crate::syntax::content::{Parseable, parser::Lexeme};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct LineBreak;

impl Parseable for LineBreak {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_char('<') && lexeme.match_next_char('\n')
    }

    fn lex(_lexeme: &Lexeme) -> LineBreak { LineBreak {} }

    fn render(&self) -> String { String::from("<br>") }

    fn flatten(&self) -> String { String::from('\n') }
}

impl std::fmt::Display for LineBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "LineBreak")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::content::parser::Token;

    #[test]
    fn token_display() {
        assert_eq!(format!("{}", Token::LineBreak(LineBreak)), "Tk:LineBreak");
    }
}
