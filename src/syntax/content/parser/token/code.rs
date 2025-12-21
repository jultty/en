use crate::{
    syntax::content::{Parseable, Lexeme},
};

pub struct Code {
    text: String,
    sticky: bool,
}

impl Parseable for Code {
    fn probe(lexeme: &Lexeme) -> bool {
        let chars = lexeme.split_chars();

        if let Some(first_char) = chars.first()
            && let Some(last_char) = chars.last()
        {
            *first_char == '`' && *last_char == '`'
        } else {
            false
        }
    }

    fn lex(lexeme: &Lexeme) -> Code {
        let sticky = [
            ",", ".", ":", ";", "!", "?", "/", "(", ")", "%", "*", "&", r#"""#,
            "'",
        ];

        Code {
            text: lexeme.text().replace("`", ""),
            sticky: sticky.contains(&lexeme.next.as_str()),
        }
    }

    fn render(&self) -> String {
        let space = if self.sticky {
            String::new()
        } else {
            String::from(" ")
        };
        format!("<code>{}</code>{space}", self.text)
    }
}
