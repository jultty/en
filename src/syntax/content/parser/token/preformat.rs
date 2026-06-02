use crate::syntax::content::{Lexeme, Parseable};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct PreFormat {
    pub text: String,
}

impl PreFormat {
    pub fn new(text: &str) -> PreFormat {
        PreFormat {
            text: String::from(text),
        }
    }
}

impl std::fmt::Display for PreFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let character_count = self.text.chars().count();
        let is_whitespace = self.text.trim_ascii().is_empty();
        let summary = if is_whitespace {
            "empty"
        } else {
            &format!("{character_count} chars")
        };
        write!(f, "PreFormat [{summary}]")
    }
}

impl Parseable for PreFormat {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_char('`') && (lexeme.next() == "\n" || lexeme.last())
    }

    fn lex(_lexeme: &Lexeme) -> PreFormat {
        panic!("Attempt to lex a preformat directly from a lexeme")
    }

    fn render(&self) -> String { format!("<pre>{}</pre>", self.text) }

    fn flatten(&self) -> String { String::default() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::content::parser::Token;

    #[test]
    #[should_panic(
        expected = "Attempt to lex a preformat directly from a lexeme"
    )]
    fn lex() {
        let lexeme = Lexeme::new("a", "b", "c");
        PreFormat::lex(&lexeme);
    }

    #[test]
    fn token_display() {
        let mut preformat = PreFormat::new("");

        assert_eq!(
            format!("{}", Token::PreFormat(preformat.clone())),
            "Tk:PreFormat [empty]"
        );

        preformat.text = "\n ".to_string();
        assert_eq!(
            format!("{}", Token::PreFormat(preformat.clone())),
            "Tk:PreFormat [empty]"
        );

        preformat.text = "text".to_string();
        assert_eq!(
            format!("{}", Token::PreFormat(preformat)),
            "Tk:PreFormat [4 chars]"
        );
    }

    #[test]
    fn flatten() {
        let preformat = PreFormat::new("");
        assert_eq!(preformat.flatten(), "");

        let token = Token::PreFormat(preformat);
        assert_eq!(token.flatten(), "");
    }
}
