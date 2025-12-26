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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::syntax::content::parser::read;

    #[test]
    fn render_anchor() {
        let anchor = Anchor::new("AnchorText", "AnchorDest", true);
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
        let anchor = Anchor::empty();
        drop(anchor.render());
    }
}
