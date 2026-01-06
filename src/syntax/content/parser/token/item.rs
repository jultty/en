use crate::syntax::content::{Parseable, Lexeme};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Item {
    open: bool,
}

impl Parseable for Item {
    fn probe(lexeme: &Lexeme) -> bool {
        (lexeme.match_as_char('-') || lexeme.match_as_char('+'))
            && lexeme.match_next_as_char(' ')
    }

    fn lex(_lexeme: &Lexeme) -> Item {
        Item { open: false }
    }

    fn render(&self) -> String {
        if self.open {
            String::from("<li>")
        } else {
            String::from("</li>")
        }
    }
}

impl Item {
    pub fn new(open: bool) -> Item {
        Item { open }
    }

    pub fn probe_end(lexeme: &Lexeme) -> bool {
        lexeme.match_as_char('\n')
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Item [{}]", if self.open { "open" } else { "closed" })
    }
}
