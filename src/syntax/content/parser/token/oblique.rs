use crate::{
    syntax::content::{Parseable, Lexeme},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Oblique {
    open: bool,
}

impl Oblique {
    pub fn new(open: bool) -> Oblique {
        Oblique { open }
    }
}

impl Parseable for Oblique {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.text() == "_"
    }

    fn lex(_lexeme: &Lexeme) -> Oblique {
        panic!("Attempt to lex an oblique tag directly from a lexeme")
    }

    fn render(&self) -> String {
        if self.open {
            String::from("<em>")
        } else {
            String::from("</em>")
        }
    }
}

impl std::fmt::Display for Oblique {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_open_state = if self.open { "open" } else { "closed" };
        write!(f, "Oblique [{display_open_state}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render() {
        let code_open = Oblique::new(true);
        assert_eq!(code_open.render(), "<em>");

        let code_closed = Oblique::new(false);
        assert_eq!(code_closed.render(), "</em>");
    }

    #[test]
    #[should_panic(
        expected = "Attempt to lex an oblique tag directly from a lexeme"
    )]
    fn lex() {
        Oblique::lex(&Lexeme::default());
    }
}
