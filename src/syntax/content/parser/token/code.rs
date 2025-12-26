use crate::{
    syntax::content::{Parseable, Lexeme},
};

#[derive(Debug)]
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
}

#[cfg(test)]
mod tests {
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
        Code::lex(&Lexeme::new("", ""));
    }
}
