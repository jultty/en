use std::{iter::Peekable, slice::Iter};

use crate::{
    prelude::*,
    syntax::content::{
        Parseable as _,
        parser::{Lexeme, State, Token, context::Block, token::PreFormat},
    },
};

/// Handles open `PreFormat` contexts until a block is fully parsed.
///
/// A return of `true` will trigger a continue in the outer parser,
/// skipping any further parsing of the current lexeme.
///
/// # Panics
/// This parser can handle only the List context, and will panic if passed an
/// unrelated context since it has no knowledge on how to handle them.
pub fn parse(
    lexeme: &Lexeme,
    state: &mut State,
    tokens: &mut Vec<Token>,
    iterator: &mut Peekable<Iter<'_, Lexeme>>,
) -> bool {
    let buffer = &mut state.buffers.preformat;
    let candidate = &mut buffer.candidate;

    #[expect(clippy::wildcard_enum_match_arm)]
    match state.context.block {
        Block::PreFormat => {
            if lexeme.match_first_char('<') {
                candidate.text.push_str("&lt;");
                candidate.text.push_str(
                    lexeme.text().strip_prefix('<').unwrap_or(&lexeme.text()),
                );
            } else if lexeme.match_last_char('>') {
                candidate.text.push_str(
                    lexeme.text().strip_suffix('>').unwrap_or(&lexeme.text()),
                );
                candidate.text.push_str("&gt;");
            } else if lexeme.match_char('\\') {
                candidate.text.push_str(lexeme.next().as_str());
                iterator.next();
                return true;
            } else if PreFormat::probe(lexeme) {
                // found end of block, push it and reset state
                log!(VERBOSE, "Accepting preformat candidate {candidate}");
                tokens.push(Token::PreFormat(candidate.clone()));
                state.context.block = Block::None;
                *candidate = PreFormat::default();
            } else {
                // anything else is pushed into the candidate preformat's text
                candidate.text.push_str(&lexeme.text());
            }
        },
        _ => {
            panic!("PreFormat context parser called for {:?}", state.context)
        },
    }
    true
}
