use crate::syntax::content::{Parseable, parser::lexeme::Lexeme};

#[derive(Debug)]
pub struct Span {
    open: Option<bool>,
}

impl Span {
    pub fn new(open: bool) -> Span {
        Span { open: Some(open) }
    }
}

impl Parseable for Span {
    fn probe(_lexeme: &Lexeme) -> bool {
        // there is no lexeme for span
        false
    }

    fn lex(_lexeme: &Lexeme) -> Span {
        Span { open: None }
    }

    fn render(&self) -> String {
        if let Some(open) = self.open {
            if open {
                "<span>".to_owned()
            } else {
                "</span>".to_owned()
            }
        } else {
            panic!("Attempt to render a span tag while open state is unknown")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe() {
        assert!(!Span::probe(&Lexeme::new(
            &crate::ONSET.elapsed().as_nanos().to_string(),
            "",
        )));
    }

    #[test]
    fn lex() {
        let span = Span::lex(&Lexeme::new(
            &crate::ONSET.elapsed().as_nanos().to_string(),
            "",
        ));
        assert!(span.open.is_none());
    }

    #[test]
    fn render() {
        let open_span = Span::new(true);
        assert_eq!(open_span.render(), "<span>");

        let closed_span = Span::new(false);
        assert_eq!(closed_span.render(), "</span>");
    }

    #[test]
    #[should_panic(
        expected = "Attempt to render a span tag while open state is unknown"
    )]
    fn render_unknown_open_state() {
        let open_span = Span::lex(&Lexeme::new("", ""));
        drop(open_span.render());
    }
}
