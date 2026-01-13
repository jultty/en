use crate::{prelude::*, graph::Graph};
use super::{TokenOutput, Parseable as _, LexMap};
use token::{LineBreak, Literal};
use context::{Block, Inline};
pub use {lexeme::Lexeme, token::Token, state::State};

pub mod token;
pub mod lexeme;
pub mod segment;
pub mod context;
pub mod point;
pub mod state;

const LEXMAP: LexMap = &[
    (LineBreak::probe, |lexeme| {
        Token::LineBreak(LineBreak::lex(lexeme))
    }),
    (Literal::probe, |lexeme| {
        Token::Literal(Literal::lex(lexeme))
    }),
];

fn lex(text: &str, map: LexMap, graph: &Graph, blocking: bool) -> TokenOutput {
    let mut tokens: Vec<Token> = Vec::default();
    let mut state = State::default();

    let segments = segment::segment(text);
    let lexemes = Lexeme::collect(&segments);

    log!("Segments: {segments:?}");

    let mut iterator = lexemes.iter().peekable();
    while let Some(lexeme) = iterator.next() {
        if lexeme.match_char('\\') {
            if let Some(next) = iterator.next() {
                tokens.push(Token::Literal(Literal::lex(next)));
            }
            continue;
        }

        if blocking {
            if context::block::parse(
                lexeme,
                &mut state,
                &mut tokens,
                &mut iterator,
                graph,
            ) {
                continue;
            }
        }

        if point::parse(lexeme, &mut state, &mut tokens, &mut iterator) {
            continue;
        }

        if context::inline::parse(
            lexeme,
            &mut state,
            &mut tokens,
            &mut iterator,
            graph,
        ) {
            continue;
        }

        for (probe, lex) in map {
            if probe(lexeme) {
                let token = lex(lexeme);
                log!("Lexmap lexed {lexeme} into {token}");
                tokens.push(token);
                break;
            }
        }
    }

    context::close(&state, &mut tokens);
    TokenOutput {
        tokens,
        format_tokens: state.format_tokens,
        text: None,
    }
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
    log!("Flattened {tokens:?} to {flat}");
    flat
}

fn parse(tokens: &[Token]) -> String {
    tokens.iter().map(Token::render).collect::<String>()
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::Graph,
        syntax::content::parser::{token::header::Level},
    };

    use super::*;

    fn read_noconfig(input: &str) -> String {
        read(input, &Graph::default())
    }

    #[test]
    fn empty_render_is_empty() {
        assert_eq!(read_noconfig(""), "");
    }

    #[test]
    fn mixed_sample() {
        let en = "`this |test|` tries ## to |brea|k|: things";
        let html = r#"<p><code>this |test|</code> tries ## to <a class="detached" title="" href="/node/k">brea</a>: things</p>"#;

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
