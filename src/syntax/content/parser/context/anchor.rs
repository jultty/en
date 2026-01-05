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
/// A return of `true` will trigger a continue in the outer parser,
/// skipping any further parsing of the current lexeme.
///
/// # Panics
/// This function will panic if can't determine the destination of an anchor.
pub fn parse(
    lexeme: &Lexeme,
    state: &mut State,
    tokens: &mut Vec<Token>,
) -> bool {
    log!("Solving: {}", state.clone().buffers.anchor);
    let buffer = &mut state.buffers.anchor;
    let candidate = &mut buffer.candidate;

    // This is only true if the anchor is leading, otherwise the outer parser
    // would already have set its text to the word before the first pipe
    if candidate.text.is_empty() {
        log!(
            "Seeking end of text at {:#?} -> {:#?}",
            lexeme.text(),
            lexeme.next()
        );
        if lexeme.next() == "|" {
            log!("End: Next lexeme is a pipe");
            buffer.text.push_str(&lexeme.text());
            candidate.text.clone_from(&buffer.text);
        } else {
            log!(
                "Pushing non-terminal {:#?} into buffer {:#?}",
                lexeme.text(),
                buffer.text
            );
            buffer.text.push_str(&lexeme.text());
        }
        return true;
    }

    if candidate.destination.is_none() {
        log!(
            "Seeking end of destination at {:#?} -> {:#?}",
            lexeme.text(),
            lexeme.next()
        );

        // Conditions in this decision tree should match the destination end
        // or some intermediary state necessary to finding it
        if lexeme.match_as_char('s')
            && lexeme.is_next_boundary()
            && !lexeme.match_next_as_char('|')
        {
            log!("End: Plural anchor");
            candidate.destination = Some(candidate.text.clone());
            candidate.text.push('s');
            if lexeme.last() {
                tokens.push(Token::Anchor(candidate.clone()));
                state.context.inline = Inline::None;
            }
            return true;
        } else if lexeme.match_as_char('|') && lexeme.is_next_boundary() {
            log!("End: Pipe followed by boundary");
            if buffer.destination.is_empty() {
                candidate.destination = Some(candidate.text.clone());
            } else {
                candidate.destination = Some(buffer.destination.clone());
            }
            tokens.push(Token::Anchor(candidate.clone()));
            state.context.inline = Inline::None;
            return true;
        } else if lexeme.match_as_char('|') && !candidate.balanced {
            log!("State: Found a pipe, but no boundary: destination follows");
            candidate.balanced = true;
            return true;
        } else if lexeme.match_as_char('|') {
            log!("End: Explicit end-of-destination pipe");
            candidate.destination = Some(buffer.destination.clone());
            return true;
        } else if !candidate.external
            && lexeme.is_punctuation()
            && lexeme.is_next_whitespace()
        {
            log!("End: Punctuation followed by whitespace");
            candidate.destination = Some(buffer.destination.clone());
            tokens.push(Token::Anchor(candidate.clone()));
            state.context.inline = Inline::None;
            return false;
        } else if lexeme.is_next_whitespace() {
            log!("End: next is whitespace");
            buffer.destination.push_str(&lexeme.text());
            candidate.destination = Some(buffer.destination.clone());
            tokens.push(Token::Anchor(candidate.clone()));
            state.context.inline = Inline::None;
            return true;

        // This else branch is the 'no end found yet' state and will keep
        // pushing lexemes into the buffer until an end is found above
        } else {
            log!(
                "Pushing non-terminal {:#?} into buffer {:#?}",
                lexeme.text(),
                buffer.destination,
            );
            if lexeme.match_as_char(':') {
                candidate.external = true;
            }
            buffer.destination.push_str(&lexeme.text());
            if lexeme.last() {
                candidate.destination = Some(buffer.destination.clone());
                tokens.push(Token::Anchor(candidate.clone()));
                state.context.inline = Inline::None;
            }
            return true;
        }
    }

    // This point should never be reached with a still None destination,
    // which would mean there is some case where the end of the destination
    // was never found and we kept filling the buffer endlessly,
    // causing the program to panic anyways when rendering anchors
    assert!(
        candidate.destination.is_some(),
        "Anchor context parsing done but no destination found: {:#?}",
        state.buffers.anchor
    );
    tokens.push(Token::Anchor(candidate.clone()));
    state.context.inline = Inline::None;
    false
}

#[cfg(test)]
mod tests {
    use crate::{syntax::content::parser, types::Graph};

    fn read_noconfig(input: &str) -> String {
        parser::read(input, &Graph::new(None).meta.config)
    }

    #[test]
    fn indifferent_trailing_pipe() {
        assert_eq!(read_noconfig("|a|a|"), read_noconfig("a|a|"));
    }

    #[test]
    fn indifferent_leading_pipe() {
        assert_eq!(read_noconfig("|a|a|"), read_noconfig("|a|a"));
    }

    #[test]
    fn indifferent_multiline_trailing_pipe() {
        assert_eq!(read_noconfig("|a|a|\nn"), read_noconfig("a|a|\nn"));
    }

    #[test]
    fn indifferent_multiline_leading_pipe() {
        assert_eq!(read_noconfig("|a|a|\nn"), read_noconfig("|a|a\nn"));
    }
}
