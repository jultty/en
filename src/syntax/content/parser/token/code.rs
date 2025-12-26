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
