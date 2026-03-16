use crate::syntax::content::{Lexeme, Parseable};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Bold {
    open: bool,
}

impl Bold {
    pub const fn new(open: bool) -> Bold { Bold { open } }
}

impl Parseable for Bold {
    fn probe(lexeme: &Lexeme) -> bool { lexeme.text() == "*" }

    fn lex(_lexeme: &Lexeme) -> Bold {
        panic!("Attempt to lex a bold tag directly from a lexeme")
    }

    fn render(&self) -> String {
        if self.open {
            String::from("<strong>")
        } else {
            String::from("</strong>")
        }
    }

    fn flatten(&self) -> String { String::default() }
}

impl std::fmt::Display for Bold {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_open_state = if self.open { "open" } else { "closed" };
        write!(f, "Bold [{display_open_state}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::content::parser::Token;

    #[test]
    fn render() {
        let code_open = Bold::new(true);
        assert_eq!(code_open.render(), "<strong>");

        let code_closed = Bold::new(false);
        assert_eq!(code_closed.render(), "</strong>");
    }

    #[test]
    #[should_panic(
        expected = "Attempt to lex a bold tag directly from a lexeme"
    )]
    fn lex() { Bold::lex(&Lexeme::default()); }

    #[test]
    fn token_display() {
        let mut bold = Bold::new(true);
        assert_eq!(format!("{}", Token::Bold(bold.clone())), "Tk:Bold [open]");

        bold.open = false;
        assert_eq!(format!("{}", Token::Bold(bold)), "Tk:Bold [closed]");
    }

    #[test]
    fn flatten() {
        let bold = Bold::new(false);
        assert_eq!(bold.flatten(), "");
    }
}
