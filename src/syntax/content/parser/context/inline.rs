use std::{iter::Peekable, slice::Iter};

use crate::syntax::content::{
    Parseable as _,
    parser::{
        context, Inline,
        lexeme::Lexeme,
        state::State,
        token::{Token, code::Code, anchor::Anchor},
    },
};

pub fn parse(
    lexeme: &Lexeme,
    state: &mut State,
    tokens: &mut Vec<Token>,
    iterator: &mut Peekable<Iter<'_, Lexeme>>,
) -> bool {
    match state.context.inline {
        Inline::None => {
            if Code::probe(lexeme) {
                state.context.inline = Inline::Code;
                tokens.push(Token::Code(Code::new(true)));
                return true;
            } else if Anchor::probe(lexeme) {
                state.context.inline = Inline::Anchor;
                state.buffers.anchor.clear();

                if lexeme.match_as_char('|') {
                    state.buffers.anchor.candidate.leading = true;
                } else {
                    state.buffers.anchor.candidate.text = lexeme.text();
                    // because we probed positively and this is not a pipe,
                    // the next lexeme must be and so it was now parsed
                    iterator.next();
                }
                return true;
            }
        },
        Inline::Code => {
            if Code::probe(lexeme) {
                state.context.inline = Inline::None;
                tokens.push(Token::Code(Code::new(false)));
                return true;
            }
        },
        Inline::Anchor => {
            if context::anchor::parse(lexeme, state, tokens) {
                return true;
            }
        },
    }
    false
}
