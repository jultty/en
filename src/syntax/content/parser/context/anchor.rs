use std::{iter::Peekable, slice::Iter};

use crate::syntax::content::parser::{
    State, context::Inline, lexeme::Lexeme, token::Token,
};

pub fn parse(
    lexeme: &Lexeme,
    iterator: &mut Peekable<Iter<'_, Lexeme>>,
    state: &mut State,
    tokens: &mut Vec<Token>,
) -> bool {
    let buffer = &mut state.buffers.anchor;
    let candidate = &mut buffer.candidate;
    if candidate.text.is_empty() {
        if lexeme.next() == "|" {
            buffer.text.push_str(&lexeme.text());
            candidate.text.clone_from(&buffer.text);
        } else {
            buffer.text.push_str(&lexeme.text());
        }
        return true;
    } else if candidate.destination.is_none() {
        // candidate is leading and we found the second pipe
        if candidate.leading && lexeme.text() == "|" {
            // third pipe immediately after second: forcing flanking
            if lexeme.match_next_first_char('|') {
                candidate.destination = Some(candidate.text.clone());
                let token = Token::Anchor(candidate.clone());
                tokens.push(token);
                state.context.inline = Inline::None;
                iterator.next();
                return true;
                // whitespace or punctuation after pipe: flanking anchor
            } else if lexeme.is_next_whitespace()
                || lexeme.is_next_punctuation()
            {
                candidate.destination = Some(candidate.text.clone());
                let token = Token::Anchor(candidate.clone());
                tokens.push(token);
                state.context.inline = Inline::None;
                // non-whitespace after pipe is the destination
            } else {
                candidate.destination = Some(lexeme.next().clone());
                let token = Token::Anchor(candidate.clone());
                tokens.push(token);
                state.context.inline = Inline::None;
                // if there is a trailing pipe, consume it
                if let Some(next) = iterator.next()
                    && next.next() == "|"
                {
                    iterator.next();
                }
            }
            // candidate is nonleading and we found a second pipe
        } else if !candidate.leading && lexeme.next() == "|" {
            candidate.destination = Some(lexeme.text());
            tokens.push(Token::Anchor(candidate.clone()));
            state.context.inline = Inline::None;
            iterator.next();
            // candidate is nonleading and we found whitespace
        } else if lexeme.is_next_whitespace() {
            candidate.destination = Some(lexeme.text());
            let token = Token::Anchor(candidate.clone());
            tokens.push(token);
            state.context.inline = Inline::None;
            // candidate is nonleading and we haven't found whitespace
        } else {
            buffer.destination.push_str(&lexeme.text());
        }
        return true;
    }
    false
}
