use crate::{
    syntax::content::{
        Parseable,
        parser::{Lexeme, token::Item},
    },
};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct List {
    pub ordered: bool,
    pub items: Vec<Item>,
}

impl Parseable for List {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_either_char('-', '+') && lexeme.match_next_char(' ')
    }

    fn lex(_lexeme: &Lexeme) -> List {
        panic!("Attempt to lex a List directly from a lexeme")
    }

    fn render(&self) -> String {
        let tag = if self.ordered { "ol" } else { "ul" };
        let mut output = String::new();
        let scale = self.scale_indent();

        let mut iterator = self.items.iter().peekable();
        while let Some(item) = iterator.next() {
            let level = item.depth.unwrap_or(0).strict_div(scale);
            let next_level = iterator
                .peek()
                .and_then(|n| n.depth)
                .unwrap_or(0)
                .strict_div(scale);

            output.push_str(&format!("<li>{}", item.text));

            if next_level > level {
                // open nested lists
                for _ in 0..(next_level.saturating_sub(level)) {
                    output.push_str(&format!("<{tag}>\n"));
                }
            } else {
                // close current item
                output.push_str("</li>");
                // close nested lists
                for _ in 0..(level.saturating_sub(next_level)) {
                    output.push_str(&format!("</{tag}></li>"));
                }
                output.push('\n');
            }
        }

        format!("\n<{tag}>\n{output}</{tag}>\n\n")
    }

    fn flatten(&self) -> String {
        format!("[List: {} items]", self.items.len())
    }
}

impl List {
    pub fn new(ordered: bool) -> List {
        List {
            ordered,
            items: vec![],
        }
    }

    fn scale_indent(&self) -> u8 {
        let width = self
            .items
            .windows(2)
            .find_map(|pair| {
                let outer = pair.first()?.depth?;
                let inner = pair.get(1)?.depth?;
                (inner > outer).then_some(inner.saturating_sub(outer))
            })
            .unwrap_or(1);

        assert!(width != 0, "Width of zero can't be a divisor");
        width
    }
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "List [{} {} items]",
            self.items.len(),
            if self.ordered { "ordered" } else { "unordered" },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::content::parser::Token;

    use super::*;

    #[test]
    fn render_flat_list() {
        let mut list = List::new(false);
        list.items = vec![
            Item::new("a", Some(0)),
            Item::new("b", Some(0)),
            Item::new("c", Some(0)),
        ];

        assert_eq!(
            list.render(),
            "\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            <li>c</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn render_nested_list() {
        let mut list = List::new(false);
        list.items = vec![
            Item::new("0Aa", Some(0)),
            Item::new("4Ba", Some(4)),
            Item::new("0Ca", Some(0)),
            Item::new("4Da", Some(4)),
            Item::new("4Db", Some(4)),
            Item::new("0Ea", Some(0)),
            Item::new("0Eb", Some(0)),
        ];

        assert_eq!(
            list.render(),
            "\n<ul>\n\
            <li>0Aa<ul>\n\
            <li>4Ba</li></ul></li>\n\
            <li>0Ca<ul>\n\
            <li>4Da</li>\n\
            <li>4Db</li></ul></li>\n\
            <li>0Ea</li>\n\
            <li>0Eb</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn render_multilevel_depth_drop() {
        let mut list = List::new(false);
        list.items = vec![
            Item::new("0Aa", Some(0)),
            Item::new("4Ba", Some(4)),
            Item::new("8Ca", Some(8)),
            Item::new("12Da", Some(12)),
            Item::new("16Ea", Some(16)),
            Item::new("8Fa", Some(8)),
            Item::new("0Ga", Some(0)),
        ];

        assert_eq!(
            list.render(),
            "\n<ul>\n\
            <li>0Aa<ul>\n\
            <li>4Ba<ul>\n\
            <li>8Ca<ul>\n\
            <li>12Da<ul>\n\
            <li>16Ea</li></ul></li></ul></li>\n\
            <li>8Fa</li></ul></li></ul></li>\n\
            <li>0Ga</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn token_display() {
        let list = List::new(false);
        assert_eq!(
            format!("{}", Token::List(list.clone())),
            "Tk:List [0 unordered items]"
        );
    }

    #[test]
    #[should_panic(expected = "Attempt to lex a List directly from a lexeme")]
    fn lex() {
        let lexeme = Lexeme::new("SL6PX", "6xsNB", "oeAHa");
        List::lex(&lexeme);
    }
}
