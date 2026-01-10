use crate::{
    syntax::content::{Parseable, Lexeme},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreFormat {
    open: Option<bool>,
}

impl PreFormat {
    pub fn new(open: bool) -> PreFormat {
        PreFormat { open: Some(open) }
    }
}

impl std::fmt::Display for PreFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_open_state = if let Some(open_state) = self.open {
            if open_state { "open" } else { "closed" }
        } else {
            "unknown"
        };
        write!(f, "PreFormat [{display_open_state}]")
    }
}

impl Parseable for PreFormat {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_first_char('`') && (lexeme.next() == "\n" || lexeme.last())
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
    use crate::syntax::content::parser::token::Token;

    use super::*;

    #[test]
    fn lex() {
        let from_empty_lexeme = PreFormat::lex(&Lexeme::default());
        assert!(from_empty_lexeme.open.is_none());

        let from_non_empty_lexeme = PreFormat::lex(&Lexeme::default());
        assert!(from_non_empty_lexeme.open.is_none());
    }

    #[test]
    #[should_panic(
        expected = "Attempt to render a preformat tag while open state is unknown"
    )]
    fn render() {
        let from_empty_lexeme = PreFormat::lex(&Lexeme::default());
        from_empty_lexeme.render();

        let from_non_empty_lexeme = PreFormat::lex(&Lexeme::default());
        from_non_empty_lexeme.render();
    }

    #[test]
    fn token_display() {
        let mut preformat = PreFormat::new(true);
        assert_eq!(
            format!("{}", Token::PreFormat(preformat.clone())),
            "Tk:PreFormat [open]"
        );

        preformat.open = Some(false);
        assert_eq!(
            format!("{}", Token::PreFormat(preformat.clone())),
            "Tk:PreFormat [closed]"
        );

        preformat.open = None;
        assert_eq!(
            format!("{}", Token::PreFormat(preformat.clone())),
            "Tk:PreFormat [unknown]"
        );
    }
}
