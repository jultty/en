use crate::{
    graph::Node,
    syntax::content::{Parseable, parser::Lexeme},
};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Anchor {
    text: String,
    destination: Option<String>,
    node_id: Option<String>,
    node: Option<Node>,
    leading: bool,
    balanced: bool,
    absolute: bool,
    external: bool,
}

impl Anchor {
    pub fn text(&self) -> String { self.text.clone() }

    pub fn set_text(&mut self, text: &str) { self.text = String::from(text); }

    pub fn text_push(&mut self, text: &str) { self.text.push_str(text); }

    pub fn destination(&self) -> Option<String> { self.destination.clone() }

    pub fn set_destination(&mut self, destination: Option<&str>) {
        self.destination = destination.map(str::to_string);
        self.route();
    }

    pub const fn balanced(&self) -> bool { self.balanced }

    pub const fn set_balanced(&mut self, balanced: bool) {
        self.balanced = balanced;
    }

    pub const fn absolute(&self) -> bool { self.absolute }

    pub const fn set_absolute(&mut self, absolute: bool) {
        self.absolute = absolute;
    }

    pub const fn external(&self) -> bool { self.external }

    pub const fn set_external(&mut self, external: bool) {
        self.external = external;
        self.absolute = true;
    }

    pub const fn set_leading(&mut self, leading: bool) {
        self.leading = leading;
    }

    pub fn node(&self) -> Option<Node> { self.node.clone() }

    pub fn set_node(&mut self, node: &Node) {
        self.node = Some(node.to_owned());
    }

    pub fn node_id(&self) -> Option<String> { self.node_id.clone() }

    pub fn set_node_id(&mut self, id: &str) {
        self.node_id = Some(id.to_owned());
    }

    fn route(&mut self) {
        self.destination = if let Some(destination) = self.destination.clone() {
            if destination.contains(':') || destination.starts_with('/') {
                Some(destination)
            } else if destination.is_empty() && self.text.is_empty() {
                None
            } else if destination.is_empty() {
                self.node_id = Some(Self::strip_fragment(&self.text));
                Some(format!("/node/{}", self.text))
            } else {
                self.node_id = self
                    .destination
                    .clone()
                    .map(|d| Anchor::strip_fragment(&d));
                Some(format!("/node/{destination}"))
            }
        } else {
            None
        }
    }

    fn strip_fragment(target: &str) -> String {
        target.split('#').next().unwrap_or(target).to_string()
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
        let Some(destination) = &self.destination else {
            panic!(
                "Attempt to render anchor {self:#?} without knowing \
                its destination."
            )
        };

        let summary = if let Some(node) = self.node.clone() {
            node.summary
        } else {
            String::default()
        };

        let classes = if self.external {
            String::from(r#"class="external""#)
        } else if self.absolute {
            String::from(r#"class="absolute""#)
        } else if self.node.is_some() {
            String::from(r#"class="attached""#)
        } else {
            String::from(r#"class="detached""#)
        };

        let text = if destination.contains('#')
            && !self.absolute
            && destination == &format!("/node/{}", self.text)
        {
            self.text
                .split('#')
                .next()
                .unwrap_or(&self.text)
                .to_string()
        } else {
            self.text.clone()
        };

        format!(
            r#"<a {classes} title="{summary}" href="{destination}">{text}</a>"#
        )
    }

    fn flatten(&self) -> String { self.text.clone() }
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use crate::dev::log::wrap;

        let wrapped_text = wrap(&self.text);
        let display_text = if wrapped_text.is_empty() {
            "<empty>"
        } else {
            wrapped_text.as_str()
        };

        let display_destination = match &self.destination {
            Some(destination) => {
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

    use super::*;
    use crate::syntax::content::parser::Token;

    #[test]
    fn render_anchor() {
        let mut anchor = Anchor::default();
        anchor.set_text("AnchorText");
        anchor.set_destination(Some("AnchorDest"));
        assert_eq!(
            anchor.render(),
            concat!(
                r#"<a class="detached" title="" "#,
                r#"href="/node/AnchorDest">AnchorText</a>"#,
            )
        );
    }

    #[test]
    #[should_panic(
        expected = "Attempt to lex an anchor directly from a lexeme"
    )]
    fn lex() { Anchor::lex(&Lexeme::default()); }

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
            format!("{}", Token::Anchor(Box::new(anchor.clone()))),
            "Tk:Anchor <empty> -> <unknown>",
        );

        anchor.text = String::from("FsJAt RTggA");
        assert_eq!(
            format!("{}", Token::Anchor(Box::new(anchor.clone()))),
            "Tk:Anchor 'FsJAt RTggA' -> <unknown>",
        );

        anchor.text = String::from("wPVo1 0OmYm");
        anchor.destination = Some(String::from("M1UEp 1gbfr"));
        assert_eq!(
            format!("{}", Token::Anchor(Box::new(anchor.clone()))),
            r#"Tk:Anchor 'wPVo1 0OmYm' -> "M1UEp 1gbfr""#,
        );

        anchor.balanced = true;
        anchor.leading = true;
        anchor.external = true;

        assert_eq!(
            format!("{}", Token::Anchor(Box::new(anchor))),
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
        assert_eq!(
            anchor.render(),
            r#"<a class="detached" title="" href="/node/BSThI">BSThI</a>"#
        );
    }

    #[test]
    fn resolve_none_destination() {
        let mut anchor = Anchor::default();
        anchor.set_destination(None);
        anchor.route(); // set_destination also called this
        assert!(anchor.destination().is_none());
    }

    #[test]
    fn set_node_id() {
        let payload = "kxBDJ0EoDVaygxpZ8NgNdQrUIBsGimTs";
        let mut anchor = Anchor::default();
        anchor.set_node_id(payload);
        assert_eq!(anchor.node_id.unwrap(), payload);
    }

    #[test]
    fn display_no_destination() {
        let anchor = Anchor::default();
        assert_eq!(format!("{anchor}"), "Anchor <empty> -> <unknown>");
    }

    #[test]
    fn flatten() {
        let payload = "tpBTViYnldoTqDsB";
        let mut anchor = Anchor::default();
        anchor.text = String::from(payload);
        assert_eq!(anchor.flatten(), payload);

        let token = Token::Anchor(Box::new(anchor));
        assert_eq!(token.flatten(), payload);
    }
}
