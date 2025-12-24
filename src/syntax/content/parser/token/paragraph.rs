use std::fmt::Display;
use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

#[derive(Debug)]
pub struct Paragraph {
    open: Option<bool>,
}

impl Paragraph {
    pub fn new(open: bool) -> Paragraph {
        Paragraph { open: Some(open) }
    }
}

impl Parseable for Paragraph {
    fn probe(lexeme: &Lexeme) -> bool {
        // lexeme for paragraph is any non-whitespace, parser knows the context
        !lexeme.is_whitespace()
    }

    fn lex(_lexeme: &Lexeme) -> Paragraph {
        Paragraph { open: None }
    }

    fn render(&self) -> String {
        if let Some(open) = self.open {
            if open {
                "<p>".to_owned()
            } else {
                "</p>".to_owned()
            }
        } else {
            panic!(
                "Attempt to render a paragraph tag while open state is unknown"
            )
        }
    }
}

impl Display for Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(open) = self.open {
            if open {
                write!(f, "Open Paragraph")
            } else {
                write!(f, "Closed Paragraph")
            }
        } else {
            write!(f, "Unitialized Paragraph (Unknown open state)")
        }
    }
}
