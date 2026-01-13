use crate::{
    prelude::*,
    syntax::content::parser::{context::Inline, Lexeme, State, Token},
    graph::Graph,
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
    graph: &Graph,
) -> bool {
    log!("Solving: {}", state.clone().buffers.anchor);
    let buffer = &mut state.buffers.anchor;
    let candidate = &mut buffer.candidate;

    // This is only true if the anchor is leading, otherwise the outer parser
    // would already have set its text to the word before the first pipe
    if candidate.text().is_empty() {
        log!(
            "Seeking end of text at {:#?} -> {:#?}",
            lexeme.text(),
            lexeme.next()
        );
        if lexeme.next() == "|" {
            log!("End: Next lexeme is a pipe");
            buffer.text.push_str(&lexeme.text());
            candidate.set_text(&buffer.text.clone());
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

    if candidate.destination().is_none() {
        log!(
            "Seeking end of destination at {:#?} -> {:#?}",
            lexeme.text(),
            lexeme.next()
        );

        // Conditions in this decision tree should match the destination end
        // or some intermediary state necessary to finding it
        if lexeme.match_char('s')
            && lexeme.is_next_boundary()
            && !lexeme.match_next_char('|')
        {
            log!("End: Plural anchor");
            candidate.set_destination(Some(&candidate.text().clone()));
            candidate.text_push("s");
            if lexeme.last() {
                push(None, tokens, state, graph);
            }
            return true;
        } else if lexeme.match_char('|') && lexeme.is_next_delimiter() {
            log!("End: Pipe followed by delimiter");
            if buffer.destination.is_empty() {
                if candidate.text().contains(':') {
                    candidate.set_external(true);
                }
                push(Some(&candidate.text().clone()), tokens, state, graph);
            } else {
                push(Some(&buffer.destination.clone()), tokens, state, graph);
            }
            return true;
        } else if lexeme.match_char('|') && !candidate.balanced() {
            log!("State: Found a pipe, but no boundary: destination follows");
            candidate.set_balanced(true);
            return true;
        } else if lexeme.match_char(':') {
            log!("State: Found a colon, marking anchor as external");
            candidate.set_external(true);
            buffer.destination.push_str(&lexeme.text());
            return true;
        } else if lexeme.match_char('|') {
            log!("End: Explicit end-of-destination pipe");
            candidate.set_destination(Some(&buffer.destination.clone()));
            return true;
        } else if !candidate.external() && lexeme.is_delimiter() {
            log!("End: Internal anchor trailed by delimiter");
            push(Some(&buffer.destination.clone()), tokens, state, graph);
            return false;
        } else if lexeme.is_next_whitespace() {
            log!("End: next is whitespace");
            buffer.destination.push_str(&lexeme.text());
            push(Some(&buffer.destination.clone()), tokens, state, graph);
            return true;
        } else if lexeme.last() {
            log!("End: end of input");
            buffer.destination.push_str(&lexeme.text());
            push(Some(&buffer.destination.clone()), tokens, state, graph);
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
                push(Some(&buffer.destination.clone()), tokens, state, graph);
            }
            return true;
        }
    }

    // This point should never be reached with a still None destination,
    // which would mean there is some case where the end of the destination
    // was never found and we kept filling the buffer endlessly,
    // causing the program to panic anyways when rendering anchors
    assert!(
        candidate.destination().is_some(),
        "Anchor context parsing done but no destination found: {:#?}",
        state.buffers.anchor
    );
    push(None, tokens, state, graph);
    false
}

fn push(
    d: Option<&str>,
    tokens: &mut Vec<Token>,
    state: &mut State,
    graph: &Graph,
) {
    let candidate = &mut state.buffers.anchor.candidate;
    if d.is_some() {
        candidate.set_destination(d);
    }

    if let Some(node_id) = candidate.node_id()
        && let Some(node) = graph.find_node(&node_id).node
    {
        candidate.set_node(&node);
    }

    tokens.push(Token::Anchor(Box::new(candidate.clone())));
    state.context.inline = Inline::None;
}

#[cfg(test)]
mod tests {
    use crate::{syntax::content::parser, graph::Graph};

    fn read(input: &str) -> String {
        parser::read(input, &Graph::default())
    }

    #[test]
    fn flanking() {
        assert_eq!(
            read("|Node|"),
            r#"<p><a class="detached" title="" href="/node/Node">Node</a></p>"#
        );
    }

    #[test]
    fn flanking_with_trailing_comma() {
        assert_eq!(
            read("|Node|,"),
            r#"<p><a class="detached" title="" href="/node/Node">Node</a>,</p>"#
        );
    }

    #[test]
    fn flanking_with_trailing_comma_and_space() {
        assert_eq!(
            read("|Node|, at"),
            r#"<p><a class="detached" title="" href="/node/Node">Node</a>, at</p>"#
        );
    }

    #[test]
    fn flanking_at_eoi() {
        assert_eq!(
            read("|Node|"),
            r#"<p><a class="detached" title="" href="/node/Node">Node</a></p>"#
        );
    }

    #[test]
    fn needless_three_pipe_anchor() {
        assert_eq!(
            read("|Node|Destination|"),
            r#"<p><a class="detached" title="" href="/node/Destination">Node</a></p>"#
        );
    }

    #[test]
    fn nonleading_second_pipe() {
        assert_eq!(
            read("Go to Node|Destination|, here"),
            r#"<p>Go to <a class="detached" title="" href="/node/Destination">Node</a>, here</p>"#,
        );
    }

    #[test]
    fn anchor_to_node_s() {
        assert_eq!(
            read("The |letter s|s|'s node: |s|!"),
            r#"<p>The <a class="detached" title="" href="/node/s">letter s</a>'s node: <a class="detached" title="" href="/node/s">s</a>!</p>"#
        );
    }

    #[test]
    fn nonleading_plural_anchor() {
        assert_eq!(
            read("The flower|s bloomed"),
            r#"<p>The <a class="detached" title="" href="/node/flower">flowers</a> bloomed</p>"#
        );
    }

    #[test]
    fn leading_plural_anchor() {
        assert_eq!(
            read("Interfaces are |element|s of |system|s."),
            r#"<p>Interfaces are <a class="detached" title="" href="/node/element">elements</a> of <a class="detached" title="" href="/node/system">systems</a>.</p>"#
        );
    }

    #[test]
    fn leading_multiword_anchor() {
        assert_eq!(
            read("interactions are |basic elements| of systems"),
            r#"<p>interactions are <a class="detached" title="" href="/node/basic elements">basic elements</a> of systems</p>"#
        );
    }

    #[test]
    fn explicit_end_of_destination() {
        assert_eq!(
            read("interactions are |basic elements|BasicElements| of systems"),
            r#"<p>interactions are <a class="detached" title="" href="/node/BasicElements">basic elements</a> of systems</p>"#
        );
    }

    #[test]
    fn explicit_end_of_external_destination() {
        assert_eq!(
            read("this |anchor example|https://example.com| is external"),
            r#"<p>this <a class="external" title="" href="https://example.com">anchor example</a> is external</p>"#
        );
    }

    #[test]
    fn anchor_destination_at_eoi() {
        assert_eq!(
            read("a |b c|d"),
            r#"<p>a <a class="detached" title="" href="/node/d">b c</a></p>"#
        );
    }

    #[test]
    fn external_anchor_destination_at_eoi() {
        assert_eq!(
            read("a b|https://example.com"),
            r#"<p>a <a class="external" title="" href="https://example.com">b</a></p>"#
        );
    }

    #[test]
    fn nonleading_plural_anchor_at_eoi() {
        assert_eq!(
            read("element|s"),
            r#"<p><a class="detached" title="" href="/node/element">elements</a></p>"#
        );
    }

    #[test]
    fn leading_plural_anchor_at_eoi() {
        assert_eq!(
            read("|element|s"),
            r#"<p><a class="detached" title="" href="/node/element">elements</a></p>"#
        );
    }

    #[test]
    fn http_external_anchor() {
        assert_eq!(
            read(
                "a |false dichotomy|https://en.wikipedia.org/wiki/False_dilemma|."
            ),
            r#"<p>a <a class="external" title="" href="https://en.wikipedia.org/wiki/False_dilemma">false dichotomy</a>.</p>"#
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
                r#"<p><a class="external" title="" href="https://rustup.rs/">Rust toolchain</a>"#,
                "\n",
                "at rustup.rs</p>",
            )
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_space() {
        assert_eq!(
            read("|Rust toolchain|https://rustup.rs/ at rustup.rs"),
            r#"<p><a class="external" title="" href="https://rustup.rs/">Rust toolchain</a> at rustup.rs</p>"#
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_eoi() {
        assert_eq!(
            read("|Rust toolchain|https://rustup.rs/"),
            r#"<p><a class="external" title="" href="https://rustup.rs/">Rust toolchain</a></p>"#
        );
    }

    #[test]
    fn newline_wrapped_anchor() {
        assert_eq!(
            read("\n|SomeAnchor|\n"),
            concat!(
                "\n",
                r#"<p><a class="detached" title="" href="/node/SomeAnchor">SomeAnchor</a></p>"#
            ),
        );
    }

    #[test]
    fn newline_separated_anchors() {
        assert_eq!(
            read("|SomeAnchor|\n|SomeOtherAnchor|\n"),
            concat!(
                r#"<p><a class="detached" title="" href="/node/SomeAnchor">SomeAnchor</a>"#,
                "\n",
                r#"<a class="detached" title="" href="/node/SomeOtherAnchor">SomeOtherAnchor</a></p>"#
            )
        );
    }

    #[test]
    fn empty_line_separated_anchors() {
        assert_eq!(
            read("|SomeAnchor|\n\n|SomeOtherAnchor|\n"),
            concat!(
                r#"<p><a class="detached" title="" href="/node/SomeAnchor">SomeAnchor</a></p>"#,
                "\n",
                "\n",
                r#"<p><a class="detached" title="" href="/node/SomeOtherAnchor">SomeOtherAnchor</a></p>"#
            ),
        );
    }

    #[test]
    fn trailing_anchor() {
        assert_eq!(
            read("see acks|acks"),
            r#"<p>see <a class="detached" title="" href="/node/acks">acks</a></p>"#
        );
    }

    #[test]
    fn trailing_anchor_with_newline() {
        assert_eq!(
            read("\nsee acks|acks\n"),
            concat!(
                "\n",
                r#"<p>see <a class="detached" title="" href="/node/acks">acks</a></p>"#
            )
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
            r#"<p>the <a class="detached" title="" href="/node/lion">lion</a>'s mouth</p>"#,
        );
    }

    #[test]
    fn anchor_with_trailing_double_quote() {
        assert_eq!(
            read(r#"the "|real|" motive"#),
            r#"<p>the "<a class="detached" title="" href="/node/real">real</a>" motive</p>"#,
        );
    }

    #[test]
    fn anchor_with_trailing_parenthesis() {
        assert_eq!(
            read("this (though |true|) was questioned"),
            r#"<p>this (though <a class="detached" title="" href="/node/true">true</a>) was questioned</p>"#,
        );
    }

    #[test]
    fn anchor_with_leading_single_quote() {
        assert_eq!(
            read("the 'real|Reality' motive"),
            r#"<p>the '<a class="detached" title="" href="/node/Reality">real</a>' motive</p>"#,
        );
    }

    #[test]
    fn anchor_with_leading_double_quote() {
        assert_eq!(
            read(r#"the "real|Reality" motive"#),
            r#"<p>the "<a class="detached" title="" href="/node/Reality">real</a>" motive</p>"#,
        );
    }

    #[test]
    fn anchor_with_leading_parenthesis() {
        assert_eq!(
            read("her (last|Surname) name"),
            r#"<p>her (<a class="detached" title="" href="/node/Surname">last</a>) name</p>"#,
        );
    }

    #[test]
    fn anchor_with_internal_apostrophe() {
        assert_eq!(
            read("the |lion's mouth|album was released"),
            r#"<p>the <a class="detached" title="" href="/node/album">lion's mouth</a> was released</p>"#
        );
    }

    #[test]
    fn nonleading_anchor_with_internal_apostrophe() {
        assert_eq!(
            read("they decided to stay at Jane's|YellowHouse that night"),
            r#"<p>they decided to stay at <a class="detached" title="" href="/node/YellowHouse">Jane's</a> that night</p>"#
        );
    }

    #[test]
    fn nonleading_anchor_with_internal_apostrophe_at_eoi() {
        assert_eq!(
            read("they decided to stay at Jane's|YellowHouse"),
            r#"<p>they decided to stay at <a class="detached" title="" href="/node/YellowHouse">Jane's</a></p>"#
        );
    }

    #[test]
    fn nonleading_anchor_with_internal_apostrophe_at_soi() {
        assert_eq!(
            read("Jane's|YellowHouse that night"),
            r#"<p><a class="detached" title="" href="/node/YellowHouse">Jane's</a> that night</p>"#
        );
    }

    #[test]
    fn anchor_with_internal_double_quotes() {
        assert_eq!(
            read(r#"the |"real"|Truth motive"#),
            r#"<p>the <a class="detached" title="" href="/node/Truth">"real"</a> motive</p>"#,
        );
    }

    #[test]
    fn anchor_with_internal_double_quotes_wrapping_spaced_words() {
        assert_eq!(
            read(r#"the |"bare reality"|Ideology they believed"#),
            r#"<p>the <a class="detached" title="" href="/node/Ideology">"bare reality"</a> they believed</p>"#,
        );
    }

    #[test]
    fn anchor_with_internal_parenthesis() {
        assert_eq!(
            read("her |last (name)|Surname was Amad"),
            r#"<p>her <a class="detached" title="" href="/node/Surname">last (name)</a> was Amad</p>"#,
        );
    }

    #[test]
    fn anchor_with_internal_parenthesis_wrapping_spaced_words() {
        assert_eq!(
            read("this |truth (though questionable) was fine|Absurd to them "),
            r#"<p>this <a class="detached" title="" href="/node/Absurd">truth (though questionable) was fine</a> to them</p>"#
        );
    }
}
