use crate::{
    syntax::content::{Parseable, Lexeme},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Code {
    open: bool,
}

impl Code {
    pub fn new(open: bool) -> Code {
        Code { open }
    }
}

impl Parseable for Code {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.text() == "`"
    }

    fn lex(_lexeme: &Lexeme) -> Code {
        panic!("Attempt to lex a code tag directly from a lexeme")
    }

    fn render(&self) -> String {
        if self.open {
            String::from("<code>")
        } else {
            String::from("</code>")
        }
    }

    fn flatten(&self) -> String {
        String::default()
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_open_state = if self.open { "open" } else { "closed" };
        write!(f, "Code [{display_open_state}]")
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::content::parser::Token;

    use super::*;

    #[test]
    fn render() {
        let code_open = Code::new(true);
        assert_eq!(code_open.render(), "<code>");

        let code_closed = Code::new(false);
        assert_eq!(code_closed.render(), "</code>");
    }

    #[test]
    #[should_panic(
        expected = "Attempt to lex a code tag directly from a lexeme"
    )]
    fn lex() {
        Code::lex(&Lexeme::default());
    }

    #[test]
    fn token_display() {
        let mut code = Code::new(true);
        assert_eq!(format!("{}", Token::Code(code.clone())), "Tk:Code [open]");

        code.open = false;
        assert_eq!(
            format!("{}", Token::Code(code.clone())),
            "Tk:Code [closed]"
        );
    }
}
