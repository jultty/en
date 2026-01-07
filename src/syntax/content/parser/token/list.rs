use std::fmt::Write as _;

use crate::{
    prelude::*,
    syntax::content::{Lexeme, Parseable, parser::token::item::Item},
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

        let indent_width = self
            .items
            .windows(2)
            .find_map(|pair| {
                let a = pair[0].depth?;
                let b = pair[1].depth?;
                (b > a).then_some(b - a)
            })
            .unwrap_or(1);

        let mut iterator = self.items.iter().peekable();

        while let Some(item) = iterator.next() {
            let current_level = item.depth.unwrap_or(0) / indent_width;
            let next_level = iterator.peek().and_then(|n| n.depth).unwrap_or(0)
                / indent_width;

            output.push_str("<li>");
            output.push_str(&item.text);

            if next_level > current_level {
                // Open nested list(s), keep <li> open
                for _ in 0..(next_level - current_level) {
                    output.push_str(&format!("<{tag}>\n"));
                }
            } else {
                // close current <li>
                output.push_str("</li>");

                // close nested lists inline
                for _ in 0..(current_level - next_level) {
                    output.push_str(&format!("</{tag}></li>"));
                }

                output.push('\n');
            }
        }

        format!("\n<{tag}>\n{output}</{tag}>\n\n")
    }
}

impl List {
    pub fn new(ordered: bool) -> List {
        List {
            ordered,
            items: vec![],
        }
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
}
