use std::fmt::Display;
use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

pub struct Anchor {
    text: String,
    destination: String,
}

impl Parseable for Anchor {
    fn probe(lexeme: &Lexeme) -> bool {
        let pipe_count = lexeme.count_char('|');
        let chars = lexeme.split_chars();
        let c1 = *match chars.first() {
            Some(c) => c,
            None => return false,
        };
        let cn = *match chars.last() {
            Some(c) => c,
            None => return false,
        };

        if !(1_i32..=3_i32).contains(&pipe_count) {
            return false;
        }
        if lexeme.to_raw().matches("||").count() > 0 {
            return false;
        }

        if pipe_count == 1 {
            c1 != '|' && cn != '|'
        } else if pipe_count == 2 {
            c1 == '|' && cn != '|'
        } else if pipe_count == 3 {
            c1 == '|' && cn == '|'
        } else {
            false
        }
    }

    fn lex(lexeme: &Lexeme) -> Anchor {
        let parts: Vec<String> = lexeme
            .to_raw()
            .split('|')
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();

        assert!(parts.len() == 2, "Parts should always be 2: {parts:?}");

        let text = parts.first().unwrap_or_else(|| unreachable!());
        let raw_destination = parts.get(1).unwrap_or_else(|| unreachable!());
        let destination =
            if raw_destination.contains(":") || raw_destination.contains("/") {
                raw_destination.to_owned()
            } else {
                format!("/node/{raw_destination}")
            };

        Anchor {
            text: text.to_owned(),
            destination,
        }
    }

    fn render(&self) -> String {
        format!(r#"<a href="{}">{}</a>"#, &self.destination, &self.text)
    }
}

impl Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Anchor: <{}> to <{}>", &self.text, &self.destination)
    }
}
