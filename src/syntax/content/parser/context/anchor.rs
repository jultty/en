use std::{iter::Peekable, slice::Iter};

use crate::{
    prelude::*,
    syntax::content::parser::{
        State, context::Inline, lexeme::Lexeme, token::Token,
    },
};

/// Handles open anchor contexts until an anchor token is fully parsed.
///
/// This function is only called if the current inline context is Anchor.
///
/// A return kind of true will trigger a continue in the outer parser,
/// skipping any further parsing of the current lexeme.
///
/// # Panics
/// This function will panic if can't determine the destination of an anchor.
pub fn parse(
    lexeme: &Lexeme,
    iterator: &mut Peekable<Iter<'_, Lexeme>>,
    state: &mut State,
    tokens: &mut Vec<Token>,
) -> bool {
    log!("Resolving open context: {:#?}", state.clone().buffers.anchor);
    let buffer = &mut state.buffers.anchor;
    let candidate = &mut buffer.candidate;

    // This is only true if the anchor is leading, otherwise the outer parser
    // would already have set its text to the word before the first pipe
    if candidate.text.is_empty() {
        log!("Seeking text at {:#?} -> {:#?}", lexeme.text(), lexeme.next());
        if lexeme.next() == "|" {
            buffer.text.push_str(&lexeme.text());
            candidate.text.clone_from(&buffer.text);
            log!("End: {:#?}", lexeme.text());
            return true;
        } else {
            log!("Pushing non-terminal {:#?} into buffer {:#?}",
                lexeme.text(), buffer.text);
            buffer.text.push_str(&lexeme.text());
            return true;
        }
    }

    if candidate.destination.is_none() {

        log!("Seeking destination at {:#?} -> {:#?}",
            lexeme.text(), lexeme.next());

        // Conditions to this decision tree should match the destination end
        if lexeme.last(){
            log!("End: no more input");
            candidate.destination = Some(candidate.text.clone());
        } else if lexeme.match_as_char('|') && lexeme.is_next_boundary() {

            if buffer.destination.is_empty() {
                candidate.destination = Some(candidate.text.clone());
            } else {
                candidate.destination = Some(buffer.destination.clone());
                return true
            }

        } else if lexeme.match_as_char('|') {
            log!("Found a pipe, but no boundary: Destination likely follows");
            return true;
        } else if lexeme.is_punctuation() && lexeme.is_next_whitespace() {
            log!("Found puncutation followed by whitespace");
            candidate.destination = Some(buffer.destination.clone());
            tokens.push(Token::Anchor(candidate.clone()));
            state.context.inline = Inline::None;
            return false;
        } else if lexeme.is_whitespace() {
            log!("End: Whitespace");
            candidate.destination = Some(buffer.destination.clone());

        // This else branch is the 'no end found yet' state and will keep
        // pushing lexemes into the buffer until an end is found above
        } else {
            log!(
                "Pushing non-terminal {:#?} into buffer {:#?}",
                lexeme.text(), buffer.destination,
            );
            buffer.destination.push_str(&lexeme.text());
            return true
        }
    }

    // This point should never be reached with a still None destination,
    // which would mean there is some case where the end of the destination
    // was never found and we kept filling the buffer endlessly,
    // causing the program to panic anyways when rendering anchors
    assert!(candidate.destination.is_some(),
        "Anchor context parsing done but no destination found: {:#?}",
        state.buffers.anchor
    );
    tokens.push(Token::Anchor(candidate.clone()));
    state.context.inline = Inline::None;
    false
}
