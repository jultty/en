use crate::syntax::content::{Parseable, parser::Lexeme};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Verse {
    open: Option<bool>,
}

impl Verse {
    pub const fn new(open: bool) -> Verse { Verse { open: Some(open) } }

    pub fn probe_end(lexeme: &Lexeme) -> bool {
        lexeme.match_char_triple('\n', '&', '\n')
    }
}

impl Parseable for Verse {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_char_triple('\n', '&', '\n')
    }

    fn lex(_lexeme: &Lexeme) -> Verse { Verse { open: None } }

    fn render(&self) -> String {
        if let Some(open) = self.open {
            if open {
                concat!("\n", r#"<p class="verse">"#).to_string()
            } else {
                "\n</p>\n".to_owned()
            }
        } else {
            panic!("Attempt to render a verse tag while open state is unknown")
        }
    }

    fn flatten(&self) -> String { String::default() }
}

impl std::fmt::Display for Verse {
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

        write!(f, "Verse [{display_open_state}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexed_verse_is_empty() {
        let verse = Verse::lex(&Lexeme::default());
        assert!(verse.open.is_none());
    }

    #[test]
    fn flat_verse_is_empty() {
        let verse = Verse::new(true);
        assert!(verse.flatten().is_empty());
    }

    #[test]
    #[should_panic(
        expected = "Attempt to render a verse tag while open state is unknown"
    )]
    fn render_attempt_with_unknown_open_state() {
        let verse = Verse::lex(&Lexeme::default());
        verse.render();
    }

    #[test]
    fn display() {
        let open = Verse::new(true);
        assert_eq!(format!("{open}"), "Verse [open]");

        let closed = Verse::new(false);
        assert_eq!(format!("{closed}"), "Verse [closed]");

        let unknown = Verse::lex(&Lexeme::default());
        assert_eq!(format!("{unknown}"), "Verse [unknown]");
    }
}
