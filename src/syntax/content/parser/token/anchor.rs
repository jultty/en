use std::fmt::Display;

use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

#[derive(Debug, Clone)]
pub struct Anchor {
    pub text: String,
    pub destination: Option<String>,
    pub leading: bool,
}

impl Parseable for Anchor {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.text() == "|" || (!lexeme.is_whitespace() && lexeme.next == "|")
    }

    fn lex(_lexeme: &Lexeme) -> Anchor {
        panic!("Attempt to lex an anchor directly from a lexeme");
    }

    fn render(&self) -> String {
        let Some(ref destination) = self.destination else {
            panic!(
                "Attempt to render anchor {self:?} without knowing its destination."
            )
        };

        format!(r#"<a href="{}">{}</a>"#, destination, &self.text)
    }
}

impl Anchor {
    pub fn new(text: &str, destination: &str, spaced: bool) -> Anchor {
        Anchor {
            text: text.to_owned(),
            destination: Some(Anchor::resolve_destination(destination)),
            leading: spaced,
        }
    }

    fn resolve_destination(raw: &str) -> String {
        if raw.contains(":") || raw.contains("/") {
            raw.to_owned()
        } else {
            format!("/node/{raw}")
        }
    }

    pub fn empty() -> Anchor {
        Anchor {
            text: String::new(),
            destination: None,
            leading: false,
        }
    }
}

impl Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Anchor: <{}> to <{:?}>", &self.text, &self.destination)
    }
}
