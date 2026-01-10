use crate::{
    syntax::content::{Parseable, Lexeme},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CheckBox {
    checked: bool,
}

impl CheckBox {
    pub fn new(checked: bool) -> CheckBox {
        CheckBox { checked }
    }
}

impl Parseable for CheckBox {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_char_triple('[', ' ', ']')
            || lexeme.match_char_triple('[', 'x', ']')
    }

    fn lex(lexeme: &Lexeme) -> CheckBox {
        use crate::prelude::*;
        log!("Lexing: {lexeme}");
        if lexeme.match_next_char('x') {
            CheckBox::new(true)
        } else {
            CheckBox::new(false)
        }
    }

    fn render(&self) -> String {
        let toggle = if self.checked { " checked " } else { "" };
        format!(r#"<input type="checkbox"{toggle}/>"#)
    }
}

impl std::fmt::Display for CheckBox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_state = if self.checked { "checked" } else { "empty" };
        write!(f, "CheckBox [{display_state}]")
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::content::parser::token::Token;

    use super::*;

    #[test]
    fn render() {
        let code_open = CheckBox::new(true);
        assert_eq!(code_open.render(), r#"<input type="checkbox" checked />"#);

        let code_closed = CheckBox::new(false);
        assert_eq!(code_closed.render(), r#"<input type="checkbox"/>"#);
    }

    #[test]
    fn token_display() {
        let mut checkbox = CheckBox::new(true);
        assert_eq!(
            format!("{}", Token::CheckBox(checkbox.clone())),
            "Tk:CheckBox [checked]"
        );

        checkbox.checked = false;
        assert_eq!(
            format!("{}", Token::CheckBox(checkbox.clone())),
            "Tk:CheckBox [empty]"
        );
    }
}
