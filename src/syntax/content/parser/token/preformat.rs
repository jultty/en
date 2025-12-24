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
        let chars = lexeme.split_chars();

        if let Some(first_char) = chars.first() {
            *first_char == '`'
        } else {
            false
        }
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
