use crate::syntax::content::{Parseable, Lexeme};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct List {
    open: bool,
    ordered: bool,
}

impl Parseable for List {
    fn probe(lexeme: &Lexeme) -> bool {
        (lexeme.match_as_char('-') || lexeme.match_as_char('+'))
            && lexeme.match_next_as_char(' ')
    }

    fn lex(_lexeme: &Lexeme) -> List {
        panic!("Attempt to lex a List directly from a lexeme")
    }

    fn render(&self) -> String {
        let bar = if self.open { "" } else { "/" };
        let tag = if self.ordered { "ol" } else { "ul" };

        format!("<{bar}{tag}>")
    }
}

impl List {
    pub fn new(open: bool, ordered: bool) -> List {
        List { open, ordered }
    }

    pub fn probe_end(lexeme: &Lexeme) -> bool {
        lexeme.match_as_char('\n')
    }
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "List [{} {}]",
            if self.open { "open" } else { "closed" },
            if self.ordered { "ordered" } else { "unordered" },
        )
    }
}
