use crate::{prelude::*, types::Config};
use super::{Parseable as _, Token, LexMap};
use token::{linebreak::LineBreak, literal::Literal};
use lexeme::Lexeme;
use context::{Block, Inline};

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

fn lex(text: &str, map: LexMap, config: &Config, blocking: bool) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::default();
    let mut state = state::State::default();

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
                config,
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
        ) {
            continue;
        }

        for &(ref probe, lex) in map {
            if probe(lexeme) {
                let token = lex(lexeme);
                log!("Lexmap lexed {lexeme} into {token}");
                tokens.push(token);
                break;
            }
        }
    }

    context::close(&state, &mut tokens);
    tokens
}

fn parse(tokens: &[Token]) -> String {
    tokens.iter().map(Token::render).collect::<String>()
}

/// Apply end-to-end point and inline parsing for nested contexts, such as
/// inside the displayed text of other tokens like anchors and list items
pub fn nest(input: &str, config: &Config) -> String {
    parse(&lex(input, LEXMAP, config, false))
}

pub(super) fn read(input: &str, config: &Config) -> String {
    parse(&lex(input, LEXMAP, config, true))
}

#[cfg(test)]
mod tests {
    use crate::{
        types::Graph,
        syntax::content::parser::{token::header::Level},
    };

    use super::*;

    fn read_noconfig(input: &str) -> String {
        read(input, &Graph::new(None).meta.config)
    }

    #[test]
    fn empty_render_is_empty() {
        assert_eq!(read_noconfig(""), "");
    }

    #[test]
    fn mixed_sample() {
        let en = "`this |test|` tries ## to |brea|k|: things";
        let html = r#"<p><code>this |test|</code> tries ## to <a href="/node/k">brea</a>: things</p>"#;

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
