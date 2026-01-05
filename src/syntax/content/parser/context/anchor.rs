use crate::{
    prelude::*,
    syntax::content::parser::{
        state::State, context::Inline, lexeme::Lexeme, token::Token,
    },
};

/// Handles open anchor contexts until an anchor token is fully parsed.
///
/// A return of `true` will trigger a continue in the outer parser,
/// skipping any further parsing of the current lexeme.
///
/// # Panics
/// If it can't determine the destination of an anchor.
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
        } else if lexeme.match_as_char('|') && lexeme.is_next_delimiter() {
            log!("End: Pipe followed by delimiter");
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
        } else if lexeme.match_as_char(':') {
            log!("State: Found a colon, marking anchor as external");
            candidate.external = true;
            buffer.destination.push_str(&lexeme.text());
            return true;
        } else if lexeme.match_as_char('|') {
            log!("End: Explicit end-of-destination pipe");
            candidate.destination = Some(buffer.destination.clone());
            return true;
        } else if !candidate.external && lexeme.is_delimiter() {
            log!("End: Internal anchor trailed by delimiter");
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
        } else if lexeme.last() {
            log!("End: end of input");
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

    fn read(input: &str) -> String {
        parser::read(input, &Graph::new(None).meta.config)
    }

    #[test]
    fn flanking_with_trailing_comma() {
        assert_eq!(read("|Node|,"), r#"<p><a href="/node/Node">Node</a>,</p>"#);
    }

    #[test]
    fn flanking_with_trailing_comma_and_space() {
        assert_eq!(
            read("|Node|, at"),
            r#"<p><a href="/node/Node">Node</a>, at</p>"#
        );
    }

    #[test]
    fn flanking_at_eoi() {
        assert_eq!(read("|Node|"), r#"<p><a href="/node/Node">Node</a></p>"#);
    }

    #[test]
    fn needless_three_pipe_anchor() {
        assert_eq!(
            read("|Node|Destination|"),
            r#"<p><a href="/node/Destination">Node</a></p>"#
        );
    }

    #[test]
    fn nonleading_second_pipe() {
        assert_eq!(
            read("Go to Node|Destination|, here"),
            r#"<p>Go to <a href="/node/Destination">Node</a>, here</p>"#,
        );
    }

    #[test]
    fn anchor_to_node_s() {
        assert_eq!(
            read("The |letter s|s|'s node: |s|!"),
            r#"<p>The <a href="/node/s">letter s</a>'s node: <a href="/node/s">s</a>!</p>"#
        );
    }

    #[test]
    fn nonleading_plural_anchor() {
        assert_eq!(
            read("The flower|s bloomed"),
            r#"<p>The <a href="/node/flower">flowers</a> bloomed</p>"#
        );
    }

    #[test]
    fn leading_plural_anchor() {
        assert_eq!(
            read("Interfaces are |element|s of |system|s."),
            r#"<p>Interfaces are <a href="/node/element">elements</a> of <a href="/node/system">systems</a>.</p>"#
        );
    }

    #[test]
    fn nonleading_plural_anchor_at_eoi() {
        assert_eq!(
            read("element|s"),
            r#"<p><a href="/node/element">elements</a></p>"#
        );
    }

    #[test]
    fn leading_plural_anchor_at_eoi() {
        assert_eq!(
            read("|element|s"),
            r#"<p><a href="/node/element">elements</a></p>"#
        );
    }

    #[test]
    fn http_external_anchor() {
        assert_eq!(
            read(
                "a |false dichotomy|https://en.wikipedia.org/wiki/False_dilemma|."
            ),
            r#"<p>a <a href="https://en.wikipedia.org/wiki/False_dilemma">false dichotomy</a>.</p>"#
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_newline() {
        assert_eq!(
            read(concat!(
                "|Rust toolchain|https://rustup.rs/",
                "\n",
                "at rustup.rs",
            )),
            concat!(
                r#"<p><a href="https://rustup.rs/">Rust toolchain</a>"#,
                "\n",
                "at rustup.rs</p>",
            )
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_space() {
        assert_eq!(
            read("|Rust toolchain|https://rustup.rs/ at rustup.rs"),
            r#"<p><a href="https://rustup.rs/">Rust toolchain</a> at rustup.rs</p>"#
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_eoi() {
        assert_eq!(
            read("|Rust toolchain|https://rustup.rs/"),
            r#"<p><a href="https://rustup.rs/">Rust toolchain</a></p>"#
        );
    }

    #[test]
    fn newline_wrapped_anchor() {
        assert_eq!(
            read("\n|SomeAnchor|\n"),
            concat!(
                "\n",
                r#"<p><a href="/node/SomeAnchor">SomeAnchor</a></p>"#
            ),
        );
    }

    #[test]
    fn newline_separated_anchors() {
        assert_eq!(
            read("|SomeAnchor|\n|SomeOtherAnchor|\n"),
            concat!(
                r#"<p><a href="/node/SomeAnchor">SomeAnchor</a>"#,
                "\n",
                r#"<a href="/node/SomeOtherAnchor">SomeOtherAnchor</a></p>"#
            )
        );
    }

    #[test]
    fn empty_line_separated_anchors() {
        assert_eq!(
            read("|SomeAnchor|\n\n|SomeOtherAnchor|\n"),
            concat!(
                r#"<p><a href="/node/SomeAnchor">SomeAnchor</a></p>"#,
                "\n",
                "\n",
                r#"<p><a href="/node/SomeOtherAnchor">SomeOtherAnchor</a></p>"#
            ),
        );
    }

    #[test]
    fn trailing_anchor() {
        assert_eq!(
            read("see acks|acks"),
            r#"<p>see <a href="/node/acks">acks</a></p>"#
        );
    }

    #[test]
    fn trailing_anchor_with_newline() {
        assert_eq!(
            read("\nsee acks|acks\n"),
            concat!("\n", r#"<p>see <a href="/node/acks">acks</a></p>"#)
        );
    }

    #[test]
    fn indifferent_trailing_pipe() {
        assert_eq!(read("|a|a|"), read("a|a|"));
    }

    #[test]
    fn indifferent_leading_pipe() {
        assert_eq!(read("|a|a|"), read("|a|a"));
    }

    #[test]
    fn indifferent_multiline_trailing_pipe() {
        assert_eq!(read("|a|a|\nn"), read("a|a|\nn"));
    }

    #[test]
    fn indifferent_multiline_leading_pipe() {
        assert_eq!(read("|a|a|\nn"), read("|a|a\nn"));
    }

    #[test]
    fn anchor_with_trailing_single_quote() {
        assert_eq!(
            read("the |lion|'s mouth"),
            r#"<p>the <a href="/node/lion">lion</a>'s mouth</p>"#,
        );
    }

    #[test]
    fn anchor_with_trailing_double_quote() {
        assert_eq!(
            read(r#"the "|real|" motive"#),
            r#"<p>the "<a href="/node/real">real</a>" motive</p>"#,
        );
    }

    #[test]
    fn anchor_with_trailing_parenthesis() {
        assert_eq!(
            read("this (though |true|) was questioned"),
            r#"<p>this (though <a href="/node/true">true</a>) was questioned</p>"#,
        );
    }

    #[test]
    fn anchor_with_leading_single_quote() {
        assert_eq!(
            read("the 'real|Reality' motive"),
            r#"<p>the '<a href="/node/Reality">real</a>' motive</p>"#,
        );
    }

    #[test]
    fn anchor_with_leading_double_quote() {
        assert_eq!(
            read(r#"the "real|Reality" motive"#),
            r#"<p>the "<a href="/node/Reality">real</a>" motive</p>"#,
        );
    }

    #[test]
    fn anchor_with_leading_parenthesis() {
        assert_eq!(
            read("her (last|Surname) name"),
            r#"<p>her (<a href="/node/Surname">last</a>) name</p>"#,
        );
    }

    #[test]
    fn anchor_with_internal_apostrophe() {
        assert_eq!(
            read("the |lion's mouth|album was released"),
            r#"<p>the <a href="/node/album">lion's mouth</a> was released</p>"#
        );
    }

    #[test]
    fn nonleading_anchor_with_internal_apostrophe() {
        assert_eq!(
            read("they decided to stay at Jane's|YellowHouse that night"),
            r#"<p>they decided to stay at <a href="/node/YellowHouse">Jane's</a> that night</p>"#
        );
    }

    #[test]
    fn nonleading_anchor_with_internal_apostrophe_at_eoi() {
        assert_eq!(
            read("they decided to stay at Jane's|YellowHouse"),
            r#"<p>they decided to stay at <a href="/node/YellowHouse">Jane's</a></p>"#
        );
    }

    #[test]
    fn nonleading_anchor_with_internal_apostrophe_at_soi() {
        assert_eq!(
            read("Jane's|YellowHouse that night"),
            r#"<p><a href="/node/YellowHouse">Jane's</a> that night</p>"#
        );
    }

    #[test]
    fn anchor_with_internal_double_quotes() {
        assert_eq!(
            read(r#"the |"real"|Truth motive"#),
            r#"<p>the <a href="/node/Truth">"real"</a> motive</p>"#,
        );
    }

    #[test]
    fn anchor_with_internal_double_quotes_wrapping_spaced_words() {
        assert_eq!(
            read(r#"the |"bare reality"|Ideology they believed"#),
            r#"<p>the <a href="/node/Ideology">"bare reality"</a> they believed</p>"#,
        );
    }

    #[test]
    fn anchor_with_internal_parenthesis() {
        assert_eq!(
            read("her |last (name)|Surname was Amad"),
            r#"<p>her <a href="/node/Surname">last (name)</a> was Amad</p>"#,
        );
    }

    #[test]
    fn anchor_with_internal_parenthesis_wrapping_spaced_words() {
        assert_eq!(
            read("this |truth (though questionable) was fine|Absurd to them "),
            r#"<p>this <a href="/node/Absurd">truth (though questionable) was fine</a> to them</p>"#
        );
    }
}
