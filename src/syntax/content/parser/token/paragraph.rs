use crate::syntax::content::{Parseable, parser::Lexeme};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Paragraph {
    open: Option<bool>,
}

impl Paragraph {
    pub const fn new(open: bool) -> Paragraph { Paragraph { open: Some(open) } }

    pub fn probe_end(lexeme: &Lexeme) -> bool {
        lexeme.match_char('\n') && lexeme.match_next_char('\n')
    }
}

impl Parseable for Paragraph {
    fn probe(lexeme: &Lexeme) -> bool {
        // lexeme for paragraph is any non-whitespace, parser knows the context
        !lexeme.is_whitespace()
    }

    fn lex(_lexeme: &Lexeme) -> Paragraph { Paragraph { open: None } }

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

    fn flatten(&self) -> String { String::default() }
}

impl std::fmt::Display for Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_open_state = match self.open {
            Some(open_state) => {
                if open_state {
                    "open"
                } else {
                    "closed"
                }
            },
            None => "unknown",
        };
        write!(f, "Paragraph [{display_open_state}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::content::parser::Token;

    #[test]
    fn lex() {
        let p = Paragraph::lex(&Lexeme::default());
        assert!(p.open.is_none());
    }

    #[test]
    #[should_panic(expected = "Attempt to render a paragraph tag while \
            open state is unknown")]
    fn render_state_unknown() {
        let p = Paragraph::lex(&Lexeme::default());
        drop(p.render());
    }

    #[test]
    fn token_display() {
        let mut paragraph = Paragraph::new(true);
        assert_eq!(
            format!("{}", Token::Paragraph(paragraph.clone())),
            "Tk:Paragraph [open]"
        );

        paragraph.open = Some(false);
        assert_eq!(
            format!("{}", Token::Paragraph(paragraph.clone())),
            "Tk:Paragraph [closed]"
        );

        paragraph.open = None;
        assert_eq!(
            format!("{}", Token::Paragraph(paragraph.clone())),
            "Tk:Paragraph [unknown]"
        );
    }
}
