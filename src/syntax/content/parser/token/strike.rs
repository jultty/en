use crate::{
    syntax::content::{Parseable, Lexeme},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Strike {
    open: bool,
}

impl Strike {
    pub fn new(open: bool) -> Strike {
        Strike { open }
    }
}

impl Parseable for Strike {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_as_char('~') && lexeme.match_next_as_char('~')
    }

    fn lex(_lexeme: &Lexeme) -> Strike {
        panic!("Attempt to lex a strike tag directly from a lexeme")
    }

    fn render(&self) -> String {
        let tag = if self.open { "<s>" } else { "</s>" };
        String::from(tag)
    }
}

impl std::fmt::Display for Strike {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_open_state = if self.open { "open" } else { "closed" };
        write!(f, "Strike [{display_open_state}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render() {
        let code_open = Strike::new(true);
        assert_eq!(code_open.render(), "<s>");

        let code_closed = Strike::new(false);
        assert_eq!(code_closed.render(), "</s>");
    }

    #[test]
    #[should_panic(
        expected = "Attempt to lex a strike tag directly from a lexeme"
    )]
    fn lex() {
        Strike::lex(&Lexeme::new("", ""));
    }
}
