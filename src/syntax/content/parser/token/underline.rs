use crate::syntax::content::{Lexeme, Parseable};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Underline {
    open: bool,
}

impl Underline {
    pub const fn new(open: bool) -> Underline { Underline { open } }
}

impl Parseable for Underline {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_char('_') && lexeme.match_next_char('_')
    }

    fn lex(_lexeme: &Lexeme) -> Underline {
        panic!("Attempt to lex an underline tag directly from a lexeme")
    }

    fn render(&self) -> String {
        if self.open {
            String::from("<u>")
        } else {
            String::from("</u>")
        }
    }

    fn flatten(&self) -> String { String::default() }
}

impl std::fmt::Display for Underline {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_open_state = if self.open { "open" } else { "closed" };
        write!(f, "Underline [{display_open_state}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::content::parser::Token;

    #[test]
    fn render() {
        let code_open = Underline::new(true);
        assert_eq!(code_open.render(), "<u>");

        let code_closed = Underline::new(false);
        assert_eq!(code_closed.render(), "</u>");
    }

    #[test]
    #[should_panic(
        expected = "Attempt to lex an underline tag directly from a lexeme"
    )]
    fn lex() { Underline::lex(&Lexeme::default()); }

    #[test]
    fn token_display() {
        let mut underline = Underline::new(true);
        assert_eq!(
            format!("{}", Token::Underline(underline.clone())),
            "Tk:Underline [open]"
        );

        underline.open = false;
        assert_eq!(
            format!("{}", Token::Underline(underline)),
            "Tk:Underline [closed]"
        );
    }

    #[test]
    fn flatten() {
        let underline = Underline::new(false);
        assert_eq!(underline.flatten(), "");

        let token = Token::Underline(underline);
        assert_eq!(token.flatten(), "");
    }
}
