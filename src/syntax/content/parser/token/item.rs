use crate::syntax::content::{Parseable, Lexeme};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Item {
    pub text: String,
    pub depth: Option<u8>,
}

impl Parseable for Item {
    fn probe(_: &Lexeme) -> bool {
        false
    }

    fn lex(_: &Lexeme) -> Item {
        panic!("Attempt to lex an item directly from a lexeme")
    }

    fn render(&self) -> String {
        panic!("Items should only be rendered by a list's render method")
    }

    fn flatten(&self) -> String {
        String::default()
    }
}

impl Item {
    pub fn new(text: &str, depth: Option<u8>) -> Item {
        Item {
            text: String::from(text),
            depth,
        }
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Item [{}] {}",
            if let Some(depth) = self.depth {
                format!("D{depth}")
            } else {
                "<unknown>".to_string()
            },
            self.text,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::content::parser::Token;

    use super::*;

    #[test]
    #[should_panic(
        expected = "Items should only be rendered by a list's render method"
    )]
    fn render() {
        let item = Item::new("aCNuZwwzrt", None);
        item.render();
    }

    #[test]
    fn probe() {
        let lexeme = Lexeme::new("bOa", "2R6", "4Mp");
        assert!(!Item::probe(&lexeme));
    }

    #[test]
    #[should_panic(expected = "Attempt to lex an item directly from a lexeme")]
    fn lex() {
        let lexeme = Lexeme::new("8kbX", "Qzqu", "iDpg");
        Item::lex(&lexeme);
    }

    #[test]
    fn token_display() {
        let mut item = Item::new("dRMy4", Some(4));
        assert_eq!(
            format!("{}", Token::Item(item.clone())),
            "Tk:Item [D4] dRMy4"
        );
        item.depth = None;
        assert_eq!(
            format!("{}", Token::Item(item.clone())),
            "Tk:Item [<unknown>] dRMy4"
        );
    }

    #[test]
    fn flatten() {
        let item = Item::new("", None);
        assert_eq!(item.flatten(), "");
    }
}
