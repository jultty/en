use crate::syntax::content::{Parseable, parser::Lexeme};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Verse {
    open: Option<bool>,
    citation: Option<String>,
}

impl Verse {
    pub fn new(open: bool) -> Verse {
        Verse {
            open: Some(open),
            citation: None,
        }
    }

    pub fn probe_end(lexeme: &Lexeme) -> bool {
        lexeme.match_char_triple('\n', '&', '\n')
    }
}

impl Parseable for Verse {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_char_triple('\n', '&', '\n')
    }

    fn lex(_lexeme: &Lexeme) -> Verse {
        Verse {
            open: None,
            citation: None,
        }
    }

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

    fn flatten(&self) -> String {
        String::default()
    }
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

        let citation = if self.citation.is_some() {
            " cited"
        } else {
            ""
        };

        write!(f, "Verse [{display_open_state}{citation}]")
    }
}
