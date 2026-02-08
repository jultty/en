use crate::syntax::content::{Parseable, parser::Lexeme};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Quote {
    open: Option<bool>,
}

impl Quote {
    pub fn new(open: bool) -> Quote {
        Quote { open: Some(open) }
    }

    pub fn probe_end(lexeme: &Lexeme) -> bool {
        lexeme.match_char_sequence('\n', '\n')
    }
}

impl Parseable for Quote {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_char('>') && lexeme.match_next_char(' ')
    }

    fn lex(_lexeme: &Lexeme) -> Quote {
        Quote { open: None }
    }

    fn render(&self) -> String {
        if let Some(open) = self.open {
            if open {
                "<blockquote>".to_owned()
            } else {
                "</blockquote>".to_owned()
            }
        } else {
            panic!("Attempt to render a quote tag while open state is unknown")
        }
    }

    fn flatten(&self) -> String {
        String::default()
    }
}

impl std::fmt::Display for Quote {
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
        write!(f, "Quote [{display_open_state}]")
    }
}
