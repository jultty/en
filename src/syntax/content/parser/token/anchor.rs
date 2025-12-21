use crate::prelude::*;

use std::fmt::Display;
use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

pub struct Anchor {
    text: String,
    destination: String,
    sticky: bool,
}

impl Parseable for Anchor {
    fn probe(lexeme: &Lexeme) -> bool {
        let pipe_count = lexeme.count_char('|');
        log!("{lexeme:?} has {pipe_count} pipes");

        if !(1..=3).contains(&pipe_count) {
            log!("Negative: Bad pipe count {pipe_count} in {lexeme:?}");
            return false;
        }
        if lexeme.text().matches("||").count() > 0 {
            log!("Negative: Contiguous pipes in {lexeme:?}");
            return false;
        }

        let parts = Anchor::split_parts(lexeme);
        if (1..=2).contains(&parts.len()) {
            log!("Positive: Parts {parts:?} with length {}", parts.len());
            true
        } else {
            log!("Negative: {parts:?} have length {}", parts.len());
            false
        }
    }

    fn lex(lexeme: &Lexeme) -> Anchor {
        let parts = Anchor::split_parts(lexeme);
        log!("Lexing anchor {parts:?}");

        let text = parts.first().unwrap_or_else(|| unreachable!());

        fn try_node_anchor(anchor: &str) -> String {
            if anchor.contains(":") || anchor.contains("/") {
                anchor.to_owned()
            } else {
                format!("/node/{anchor}")
            }
        }

        let destination = match parts.get(1) {
            Some(d) => try_node_anchor(d),
            None => try_node_anchor(text),
        };

        let sticky = [
            ",", ".", ":", ";", "!", "?", "/", "(", ")", "%", "*", "&", r#"""#,
            "'",
        ];

        log!("Lexed anchor: {text} -> {destination}");
        Anchor {
            text: text.to_owned(),
            destination,
            sticky: sticky.contains(&lexeme.next.as_str()),
        }
    }

    fn render(&self) -> String {
        let space = if self.sticky {
            String::new()
        } else {
            String::from(" ")
        };
        format!(
            r#"<a href="{}">{}</a>{space}"#,
            &self.destination, &self.text
        )
    }
}

impl Anchor {
    fn split_parts(lexeme: &Lexeme) -> Vec<String> {
        lexeme
            .text()
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    }
}

impl Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Anchor: <{}> to <{}>", &self.text, &self.destination)
    }
}
