use crate::{
    syntax::content::{Parseable, parser::lexeme::Lexeme},
    types::Node,
};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Anchor {
    text: String,
    destination: Option<String>,
    node: Option<Node>,
    leading: bool,
    balanced: bool,
    external: bool,
}

impl Anchor {
    pub fn new(
        text: &str,
        destination: &str,
        node: Option<Node>,
        leading: bool,
        external: bool,
        balanced: bool,
    ) -> Anchor {
        let mut anchor = Anchor {
            text: text.to_owned(),
            destination: Some(String::from(destination)),
            node,
            leading,
            external,
            balanced,
        };

        anchor.route();
        anchor
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = String::from(text);
    }

    pub fn text_push(&mut self, text: &str) {
        self.text.push_str(text);
    }

    pub fn destination(&self) -> Option<String> {
        self.destination.clone()
    }

    pub fn set_destination(&mut self, destination: Option<&str>) {
        self.destination = destination.map(str::to_string);
        self.route();
    }

    pub fn balanced(&self) -> bool {
        self.balanced
    }

    pub fn set_balanced(&mut self, balanced: bool) {
        self.balanced = balanced;
    }

    pub fn external(&self) -> bool {
        self.external
    }

    pub fn set_external(&mut self, external: bool) {
        self.external = external;
    }

    pub fn set_leading(&mut self, leading: bool) {
        self.leading = leading;
    }

    fn route(&mut self) {
        self.destination = if let Some(destination) = self.destination.clone() {
            if destination.contains(":") || destination.contains("/") {
                Some(destination)
            } else if destination.is_empty() && self.text.is_empty() {
                None
            } else if destination.is_empty() {
                Some(format!("/node/{}", self.text))
            } else {
                Some(format!("/node/{destination}"))
            }
        } else {
            None
        }
    }
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

        format!(r#"<a href="{}">{}</a>"#, destination, &self.text)
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

#[cfg(test)]
mod tests {

    use crate::syntax::content::parser::token::Token;

    use super::*;

    #[test]
    fn render_anchor() {
        let mut anchor = Anchor::default();
        anchor.set_text("AnchorText");
        anchor.set_destination(Some("AnchorDest"));
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

    #[test]
    fn display_empty_destination() {
        let mut anchor = Anchor::default();
        anchor.set_destination(Some(""));
        assert_eq!(format!("{anchor}"), "Anchor <empty> -> <unknown>");
    }

    #[test]
    fn render_empty_destination() {
        let mut anchor = Anchor::default();
        anchor.set_text("BSThI");
        anchor.set_destination(Some(""));
        assert_eq!(anchor.render(), r#"<a href="/node/BSThI">BSThI</a>"#);
    }

    #[test]
    fn resolve_none_destination() {
        let mut anchor = Anchor::default();
        anchor.set_destination(None);
        anchor.route(); // set_destination also called this
        assert!(anchor.destination().is_none());
    }
}
