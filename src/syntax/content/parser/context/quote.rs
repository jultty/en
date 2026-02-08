use std::{iter::Peekable, slice::Iter};

use crate::{
    graph::Graph,
    prelude::*,
    syntax::content::parser::{
        Lexeme, State, Token, context::Block, format, state, token::Quote,
    },
};

/// Handles open quote contexts until a quote is fully parsed.
///
/// A return of `true` will trigger a continue in the outer parser,
/// skipping any further parsing of the current lexeme.
///
/// # Panics
/// This parser can handle only the Quote context, and will panic if passed an
/// unrelated context since it has no knowledge on how to handle them.
pub fn parse(
    lexeme: &Lexeme,
    state: &mut State,
    tokens: &mut Vec<Token>,
    iterator: &mut Peekable<Iter<'_, Lexeme>>,
    graph: &Graph,
) -> bool {
    let buffer = &mut state.buffers.quote;
    let candidate = &mut buffer.candidate;

    #[allow(clippy::wildcard_enum_match_arm)]
    match state.context.block {
        Block::Quote => {
            if Quote::probe_end(lexeme) {
                log!("Probed end of quote on {lexeme}");
                let (text, text_tokens) = format(&candidate.text, graph);
                candidate.text = text;
                state.format_tokens.extend_from_slice(&text_tokens);

                if let Some(citation) = &candidate.citation {
                    let (formatted_citation, citation_tokens) =
                        format(citation, graph);
                    candidate.citation = Some(formatted_citation);
                    state.format_tokens.extend_from_slice(&citation_tokens);

                    if let Some(url) = citation_tokens.into_iter().find_map(
                        |token| match token {
                            Token::Anchor(a) if a.external() => a.destination(),
                            _ => None,
                        },
                    ) {
                        candidate.url = Some(url);
                    }
                }

                tokens.push(Token::Quote(candidate.clone()));
                log!(VERBOSE, "Block Context: Quote -> None on {lexeme}");
                state.context.block = Block::None;
                *buffer = state::QuoteBuffer::default();
            } else if !buffer.in_citation
                && lexeme.match_char('\n')
                && lexeme.next() == "--"
            {
                log!("Matched citation start on {lexeme}");
                buffer.in_citation = true;
                iterator.next();
                iterator.next();
            } else if lexeme.match_char_sequence('\n', '>') {
                log!("Matched break-aware sequence on {lexeme}");
                candidate.text.push_str(" <\n");
                iterator.next();
            } else {
                log!("Entered quote else branch on {lexeme}");
                if buffer.in_citation {
                    log!("Extending citation on {lexeme}");
                    candidate.extend_citation(&lexeme.text());
                    if lexeme.match_char('\n') && lexeme.next() == "--" {
                        candidate.text.push('\n');
                        iterator.next();
                    } else if lexeme.match_char('\n') {
                        buffer.in_citation = false;
                    }
                } else {
                    log!("Extending quote on {lexeme}");
                    candidate.text.push_str(&lexeme.text());
                }
            }
        },
        _ => {
            panic!("Quote context parser called to handle non-quote context")
        },
    }
    true
}
