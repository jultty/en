use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Anchor {
    pub text: String,
    pub destination: Option<String>,
    pub leading: bool,
    pub balanced: bool,
    pub external: bool,
}

impl Parseable for Anchor {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.text() == "|"
            || (!lexeme.is_whitespace() && lexeme.next() == "|")
    }

    fn lex(_lexeme: &Lexeme) -> Anchor {
        panic!("Attempt to lex an anchor directly from a lexeme");
    }

    fn render(&self) -> String {
        let Some(ref destination) = self.destination else {
            panic!(
                "Attempt to render anchor {self:#?} without knowing its destination."
            )
        };

        let non_empty_destination = if destination.is_empty() {
            self.text.clone()
        } else {
            destination.to_owned()
        };

        format!(
            r#"<a href="{}">{}</a>"#,
            Anchor::resolve_destination(&non_empty_destination),
            &self.text
        )
    }
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use crate::dev::wrap;

        let display_destination = match self.destination {
            Some(ref destination) => {
                if destination.is_empty() {
                    "<empty>"
                } else {
                    destination
                }
            },
            None => "<unknown>",
        };

        let mut tail = String::new();

        if self.leading {
            tail.push_str(" [Leading]");
        }
        if self.balanced {
            tail.push_str(" [Balanced]");
        }
        if self.external {
            tail.push_str(" [External]");
        }

        write!(
            f,
            "Anchor {:?} -> {:?}{}",
            wrap(&self.text),
            display_destination,
            tail
        )
    }
}

impl Anchor {
    pub fn new(
        text: &str,
        destination: &str,
        leading: bool,
        external: bool,
        balanced: bool,
    ) -> Anchor {
        Anchor {
            text: text.to_owned(),
            destination: Some(Anchor::resolve_destination(destination)),
            leading,
            external,
            balanced,
        }
    }

    fn resolve_destination(raw: &str) -> String {
        if raw.contains(":") || raw.contains("/") {
            raw.to_owned()
        } else {
            format!("/node/{raw}")
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn render_anchor() {
        let anchor =
            Anchor::new("AnchorText", "AnchorDest", true, false, false);
        assert_eq!(
            anchor.render(),
            r#"<a href="/node/AnchorDest">AnchorText</a>"#
        );
    }

    #[test]
    #[should_panic(
        expected = "Attempt to lex an anchor directly from a lexeme"
    )]
    fn lex() {
        Anchor::lex(&Lexeme::new("", ""));
    }

    #[test]
    #[should_panic(expected = "without knowing its destination")]
    fn unknown_destination_render() {
        let anchor = Anchor::default();
        drop(anchor.render());
    }
}
