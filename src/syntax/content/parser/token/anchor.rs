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
            || ((!lexeme.is_whitespace() && !lexeme.is_delimiter())
                && lexeme.next() == "|")
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

        let wrapped_text = wrap(&self.text);
        let display_text = if wrapped_text.is_empty() {
            "<empty>"
        } else {
            wrapped_text.as_str()
        };

        let display_destination = match self.destination {
            Some(ref destination) => {
                if destination.is_empty() {
                    String::from("<empty>")
                } else {
                    format!("{destination:?}")
                }
            },
            None => String::from("<unknown>"),
        };

        let mut tail = String::default();

        if self.leading {
            tail.push_str(" +Leading");
        }
        if self.balanced {
            tail.push_str(" +Balanced");
        }
        if self.external {
            tail.push_str(" +External");
        }

        write!(f, "Anchor {display_text} -> {display_destination}{tail}")
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

    use crate::syntax::content::parser::token::Token;

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
        Anchor::lex(&Lexeme::default());
    }

    #[test]
    #[should_panic(expected = "without knowing its destination")]
    fn unknown_destination_render() {
        let anchor = Anchor::default();
        drop(anchor.render());
    }

    #[test]
    fn token_display() {
        let mut anchor = Anchor::default();
        assert_eq!(
            format!("{}", Token::Anchor(anchor.clone())),
            "Tk:Anchor <empty> -> <unknown>",
        );

        anchor.text = String::from("FsJAt RTggA");
        assert_eq!(
            format!("{}", Token::Anchor(anchor.clone())),
            "Tk:Anchor 'FsJAt RTggA' -> <unknown>",
        );

        anchor.text = String::from("wPVo1 0OmYm");
        anchor.destination = Some(String::from("M1UEp 1gbfr"));
        assert_eq!(
            format!("{}", Token::Anchor(anchor.clone())),
            r#"Tk:Anchor 'wPVo1 0OmYm' -> "M1UEp 1gbfr""#,
        );

        anchor.balanced = true;
        anchor.leading = true;
        anchor.external = true;

        assert_eq!(
            format!("{}", Token::Anchor(anchor.clone())),
            "Tk:Anchor 'wPVo1 0OmYm' -> \"M1UEp 1gbfr\" \
            +Leading +Balanced +External",
        );
    }
}
