use std::fmt::Display;
use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

pub struct Span {
    open: Option<bool>,
}

impl Span {
    pub fn new(open: bool) -> Span {
        Span { open: Some(open) }
    }
}

impl Parseable for Span {
    fn probe(_lexeme: &Lexeme) -> bool {
        // there is no lexeme for span
        false
    }

    fn lex(_lexeme: &Lexeme) -> Span {
        Span { open: None }
    }

    fn render(&self) -> String {
        if let Some(open) = self.open {
            if open {
                "<span>".to_owned()
            } else {
                "</span>".to_owned()
            }
        } else {
            panic!("Attempt to render a span tag while open state is unknown")
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(open) = self.open {
            if open {
                write!(f, "Open Span")
            } else {
                write!(f, "Closed Span")
            }
        } else {
            write!(f, "Span (Unknown open state)")
        }
    }
}
