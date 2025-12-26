use crate::{
    syntax::content::{Parseable, Lexeme},
};

#[derive(Debug)]
pub struct PreFormat {
    open: Option<bool>,
}

impl PreFormat {
    pub fn new(open: bool) -> PreFormat {
        PreFormat { open: Some(open) }
    }
}

impl Parseable for PreFormat {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_first_char('`') && lexeme.next == "\n"
    }

    fn lex(_lexeme: &Lexeme) -> PreFormat {
        PreFormat { open: None }
    }

    fn render(&self) -> String {
        if let Some(o) = self.open {
            if o {
                "<pre>".to_owned()
            } else {
                "</pre>".to_owned()
            }
        } else {
            panic!(
                "Attempt to render a preformat tag while open state is unknown"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex() {
        let from_empty_lexeme = PreFormat::lex(&Lexeme::new("", ""));
        assert!(from_empty_lexeme.open.is_none());

        let from_non_empty_lexeme = PreFormat::lex(&Lexeme::new("`", "`"));
        assert!(from_non_empty_lexeme.open.is_none());
    }

    #[test]
    #[should_panic(
        expected = "Attempt to render a preformat tag while open state is unknown"
    )]
    fn render() {
        let from_empty_lexeme = PreFormat::lex(&Lexeme::new("", ""));
        from_empty_lexeme.render();

        let from_non_empty_lexeme = PreFormat::lex(&Lexeme::new("`", "`"));
        from_non_empty_lexeme.render();
    }
}
