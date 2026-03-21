use crate::syntax::content::{Parseable, parser::Lexeme};

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

    fn render(&self) -> String { self.text.clone() }

    fn flatten(&self) -> String { self.text.clone() }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Literal {}", crate::dev::log::wrap(&self.text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::content::parser::Token;

    #[test]
    fn token_display() {
        let mut literal = Literal {
            text: String::from("MYpDT"),
        };
        assert_eq!(
            format!("{}", Token::Literal(literal.clone())),
            "Tk:Literal MYpDT"
        );

        literal.text = String::from("TjY02");
        assert_eq!(format!("{}", Token::Literal(literal)), "Tk:Literal TjY02");
    }

    #[test]
    fn flatten() {
        let payload = "vJtsvWD7ErYB";
        let literal = Literal::lex(&Lexeme::new(payload, "", ""));
        assert_eq!(literal.flatten(), payload);
        assert_eq!(Token::Literal(literal).flatten(), payload);
    }
}
