use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Paragraph {
    open: Option<bool>,
}

impl Paragraph {
    pub fn new(open: bool) -> Paragraph {
        Paragraph { open: Some(open) }
    }

    pub fn probe_end(lexeme: &Lexeme) -> bool {
        lexeme.match_as_char('\n') && lexeme.match_next_as_char('\n')
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex() {
        let p = Paragraph::lex(&Lexeme::new("", ""));
        assert!(p.open.is_none());
    }

    #[test]
    #[should_panic(
        expected = "Attempt to render a paragraph tag while open state is unknown"
    )]
    fn render_state_unknown() {
        let p = Paragraph::lex(&Lexeme::new("", ""));
        drop(p.render());
    }
}
