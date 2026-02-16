use context::{Block, Inline};
pub use lexeme::Lexeme;
use lexer::{LEXMAP, lex};
pub use state::State;
pub use token::Token;

use crate::{graph::Graph, prelude::*, syntax::content::TokenOutput};

pub mod context;
pub mod lexeme;
pub mod lexer;
pub mod point;
pub mod segment;
pub mod state;
pub mod token;

fn parse(tokens: &[Token]) -> String {
    tokens.iter().map(Token::render).collect::<String>()
}

pub(super) fn read(input: &str, graph: &Graph) -> String {
    parse(&lex(input, LEXMAP, graph, true).tokens)
}

pub(super) fn rich_read(input: &str, graph: &Graph) -> TokenOutput {
    let lex_output = lex(input, LEXMAP, graph, true);
    let text = parse(&lex_output.tokens);
    TokenOutput {
        text: Some(text),
        tokens: lex_output.tokens,
        format_tokens: lex_output.format_tokens,
    }
}

/// Apply end-to-end point and inline parsing for nested formatting, such as
/// inside the display text of anchors and list items
pub fn format(input: &str, graph: &Graph) -> (String, Vec<Token>) {
    let tokens = lex(input, LEXMAP, graph, false).tokens;
    (parse(&tokens), tokens)
}

// Strip special syntax for display in noninteractive or plain-text display
pub fn flatten(input: &str, graph: &Graph) -> String {
    let tokens = lex(input, LEXMAP, graph, true).tokens;
    let flat = tokens.iter().map(Token::flatten).collect::<String>();
    log!(VERBOSE, "Flattened {tokens:?} to {flat}");
    flat
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graph::Graph, syntax::content::parser::token::header::Level};

    fn read_noconfig(input: &str) -> String { read(input, &Graph::default()) }

    #[test]
    fn empty_render_is_empty() {
        assert_eq!(read_noconfig(""), "");
    }

    #[test]
    fn mixed_sample() {
        let en = "`this |test|` tries ## to |brea|k|: things";
        let html = concat!(
            r#"<p><code>this |test|</code> tries ## to <a "#,
            r#"class="detached" title="" href="/node/k">brea</a>: things</p>"#,
        );

        assert_eq!(read_noconfig(en), html);
    }

    #[test]
    fn display_level() {
        assert_eq!(format!("{}", Level::One), "1");
        assert_eq!(format!("{}", Level::Two), "2");
        assert_eq!(format!("{}", Level::Three), "3");
        assert_eq!(format!("{}", Level::Four), "4");
        assert_eq!(format!("{}", Level::Five), "5");
        assert_eq!(format!("{}", Level::Six), "6");
    }
}
