use std::{iter::Peekable, slice::Iter};

use crate::{
    prelude::*,
    syntax::content::{
        Parseable as _,
        parser::{
            Inline, context,
            lexeme::Lexeme,
            state::{AnchorBuffer, State},
            token::{Token, anchor::Anchor, code::Code, literal::Literal},
        },
    },
    types::Graph,
};

pub fn parse(
    lexeme: &Lexeme,
    state: &mut State,
    tokens: &mut Vec<Token>,
    iterator: &mut Peekable<Iter<'_, Lexeme>>,
    graph: &Graph,
) -> bool {
    match state.context.inline {
        Inline::None => {
            if Code::probe(lexeme) {
                log!("Inline Context: None -> Code on {lexeme}");
                state.context.inline = Inline::Code;
                tokens.push(Token::Code(Code::new(true)));
                return true;
            } else if Anchor::probe(lexeme) {
                log!("Inline Context: None -> Anchor on {lexeme}");
                state.context.inline = Inline::Anchor;
                state.buffers.anchor = AnchorBuffer::default();

                if lexeme.match_char('|') {
                    state.buffers.anchor.candidate.set_leading(true);
                } else {
                    state.buffers.anchor.candidate.set_text(&lexeme.text());
                    // because we probed positively and this is not a pipe,
                    // the next lexeme must be and so it was now parsed
                    iterator.next();
                }
                return true;
            }
        },
        Inline::Code => {
            if Code::probe(lexeme) {
                log!("Inline Context: Code -> None on {lexeme}");
                state.context.inline = Inline::None;
                tokens.push(Token::Code(Code::new(false)));
                return true;
            } else {
                tokens.push(Token::Literal(Literal::lex(lexeme)));
                return true;
            }
        },
        Inline::Anchor => {
            if context::anchor::parse(lexeme, state, tokens, graph) {
                return true;
            }
        },
    }
    false
}
