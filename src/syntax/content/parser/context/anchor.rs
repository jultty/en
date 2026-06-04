use crate::{
    graph::Graph,
    prelude::*,
    syntax::content::parser::{Lexeme, State, Token, context::Inline},
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
    log!(VERBOSE, "Solving: {}", state.clone().buffers.anchor);
    let buffer = &mut state.buffers.anchor;
    let candidate = &mut buffer.candidate;

    // This is only true if the anchor is leading, otherwise the outer parser
    // would already have set its text to the word before the first pipe
    if candidate.text().is_empty() {
        log!(
            VERBOSE,
            "Seeking end of text at {:#?} -> {:#?}",
            lexeme.text(),
            lexeme.next()
        );
        if lexeme.next() == "|" {
            log!(VERBOSE, "End: Next lexeme is a pipe");
            buffer.text.push_str(&lexeme.text());
            candidate.set_text(&buffer.text.clone());
            if buffer.text.starts_with('/') {
                candidate.set_absolute(true);
            }
        } else {
            log!(
                VERBOSE,
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
            VERBOSE,
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
            log!(VERBOSE, "End: Plural anchor");
            candidate.set_destination(Some(&candidate.text()));
            candidate.text_push("s");
            if lexeme.last() {
                push(None, tokens, state, graph);
            }
            return true;
        } else if lexeme.match_char('|') && lexeme.is_next_delimiter() {
            log!(VERBOSE, "End: Pipe followed by delimiter");
            if buffer.destination.is_empty() {
                if candidate.text().contains(':') {
                    candidate.set_external(true);
                }
                push(Some(&candidate.text()), tokens, state, graph);
            } else {
                push(Some(&buffer.destination.clone()), tokens, state, graph);
            }
            return true;
        } else if lexeme.match_char('|') && !candidate.balanced() {
            log!(
                VERBOSE,
                "State: Found a pipe, but no boundary: destination follows"
            );
            candidate.set_balanced(true);
            if lexeme.match_next_first_char('/') {
                log!(
                    VERBOSE,
                    "State: Destination starts with a dash, marking as absolute"
                );
                candidate.set_absolute(true);
            }
            return true;
        } else if lexeme.match_char(':') {
            log!(VERBOSE, "State: Found a colon, marking anchor as external");
            candidate.set_external(true);
            buffer.destination.push_str(&lexeme.text());
            return true;
        } else if lexeme.match_char('|') {
            log!(VERBOSE, "End: Explicit end-of-destination pipe");
            candidate.set_destination(Some(&buffer.destination.clone()));
            return true;
        } else if !candidate.external() && lexeme.is_delimiter() {
            log!(VERBOSE, "End: Internal anchor trailed by delimiter");
            push(Some(&buffer.destination.clone()), tokens, state, graph);
            return false;
        } else if lexeme.is_next_whitespace() {
            log!(VERBOSE, "End: next is whitespace");
            buffer.destination.push_str(&lexeme.text());
            push(Some(&buffer.destination.clone()), tokens, state, graph);
            return true;
        } else if lexeme.last() {
            log!(VERBOSE, "End: end of input");
            buffer.destination.push_str(&lexeme.text());
            push(Some(&buffer.destination.clone()), tokens, state, graph);
            return true;

        // This else branch is the 'no end found yet' state and will keep
        // pushing lexemes into the buffer until an end is found above
        } else {
            log!(
                VERBOSE,
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
    use crate::{graph::Graph, syntax::content::parser};

    fn read(input: &str) -> String { parser::read(input, &Graph::default()) }

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
            concat!(
                r#"<p><a class="detached" title="" "#,
                r#"href="/node/Node">Node</a>, at</p>"#,
            )
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
            concat!(
                r#"<p><a class="detached" title="" "#,
                r#"href="/node/Destination">Node</a></p>"#
            )
        );
    }

    #[test]
    fn nonleading_second_pipe() {
        assert_eq!(
            read("Go to Node|Destination|, here"),
            concat!(
                r#"<p>Go to <a class="detached" title="" "#,
                r#"href="/node/Destination">Node</a>, here</p>"#
            ),
        );
    }

    #[test]
    fn anchor_to_node_s() {
        assert_eq!(
            read("The |letter s|s|'s node: |s|!"),
            concat!(
                r#"<p>The <a class="detached" title="" "#,
                r#"href="/node/s">letter s</a>'s node: "#,
                r#"<a class="detached" title="" href="/node/s">s</a>!</p>"#
            )
        );
    }

    #[test]
    fn nonleading_plural_anchor() {
        assert_eq!(
            read("The flower|s bloomed"),
            concat!(
                r#"<p>The <a class="detached" title="" "#,
                r#"href="/node/flower">flowers</a> bloomed</p>"#,
            )
        );
    }

    #[test]
    fn leading_plural_anchor() {
        assert_eq!(
            read("Interfaces are |element|s of |system|s."),
            concat!(
                r#"<p>Interfaces are <a class="detached" title="" "#,
                r#"href="/node/element">elements</a> of <a class="detached" "#,
                r#"title="" href="/node/system">systems</a>.</p>"#
            )
        );
    }

    #[test]
    fn leading_multiword_anchor() {
        assert_eq!(
            read("interactions are |basic elements| of systems"),
            concat!(
                r#"<p>interactions are <a class="detached" title="""#,
                r#" href="/node/basic elements">basic elements</a> "#,
                r#"of systems</p>"#,
            ),
        );
    }

    #[test]
    fn explicit_end_of_destination() {
        assert_eq!(
            read("interactions are |basic elements|BasicElements| of systems"),
            concat!(
                r#"<p>interactions are <a class="detached" title="" "#,
                r#"href="/node/BasicElements">basic elements</a> of "#,
                r#"systems</p>"#
            )
        );
    }

    #[test]
    fn explicit_end_of_external_destination() {
        assert_eq!(
            read("this |anchor example|https://example.com| is external"),
            concat!(
                r#"<p>this <a class="external" title="" "#,
                r#"href="https://example.com">anchor example</a> is "#,
                r#"external</p>"#
            )
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
            concat!(
                r#"<p>a <a class="external" title="" "#,
                r#"href="https://example.com">b</a></p>"#,
            )
        );
    }

    #[test]
    fn nonleading_plural_anchor_at_eoi() {
        assert_eq!(
            read("element|s"),
            concat!(
                r#"<p><a class="detached" title="" "#,
                r#"href="/node/element">elements</a></p>"#,
            )
        );
    }

    #[test]
    fn leading_plural_anchor_at_eoi() {
        assert_eq!(
            read("|element|s"),
            concat!(
                r#"<p><a class="detached" title="" "#,
                r#"href="/node/element">elements</a></p>"#,
            )
        );
    }

    #[test]
    fn absolute_anchor() {
        let parse_result =
            parser::rich_read("see the |raw endpoints|/data|.", &Graph::load());
        println!("Parsed tokens: {:#?}", parse_result.tokens);
        assert_eq!(
            parse_result.text.unwrap(),
            concat!(
                r#"<p>see the <a class="absolute" title="" "#,
                r#"href="/data">"#,
                r#"raw endpoints</a>.</p>"#,
            ),
        );
    }

    #[test]
    fn http_external_anchor() {
        assert_eq!(
            read("a |false dichotomy|https://wikipedia.org/False_dilemma|."),
            concat!(
                r#"<p>a <a class="external" title="" "#,
                r#"href="https://wikipedia.org/False_dilemma">"#,
                r#"false dichotomy</a>.</p>"#,
            ),
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
                r#"<p><a class="external" title="" "#,
                r#"href="https://rustup.rs/">Rust toolchain</a>"#,
                "\n",
                "at rustup.rs</p>",
            )
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_space() {
        assert_eq!(
            read("|Rust toolchain|https://rustup.rs/ at rustup.rs"),
            concat!(
                r#"<p><a class="external" title="" "#,
                r#"href="https://rustup.rs/">Rust toolchain</a> "#,
                r#"at rustup.rs</p>"#,
            ),
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_eoi() {
        assert_eq!(
            read("|Rust toolchain|https://rustup.rs/"),
            concat!(
                r#"<p><a class="external" title="" "#,
                r#"href="https://rustup.rs/">Rust toolchain</a></p>"#,
            )
        );
    }

    #[test]
    fn newline_wrapped_anchor() {
        assert_eq!(
            read("\n|SomeAnchor|\n"),
            concat!(
                "\n",
                concat!(
                    r#"<p><a class="detached" title="" "#,
                    r#"href="/node/SomeAnchor">SomeAnchor</a></p>"#,
                )
            ),
        );
    }

    #[test]
    fn newline_separated_anchors() {
        assert_eq!(
            read("|SomeAnchor|\n|SomeOtherAnchor|\n"),
            concat!(
                r#"<p><a class="detached" title="" "#,
                r#"href="/node/SomeAnchor">SomeAnchor</a>"#,
                "\n",
                r#"<a class="detached" title="" "#,
                r#"href="/node/SomeOtherAnchor">SomeOtherAnchor</a></p>"#,
            )
        );
    }

    #[test]
    fn empty_line_separated_anchors() {
        assert_eq!(
            read("|SomeAnchor|\n\n|SomeOtherAnchor|\n"),
            concat!(
                r#"<p><a class="detached" title="" "#,
                r#"href="/node/SomeAnchor">SomeAnchor</a></p>"#,
                "\n",
                "\n",
                r#"<p><a class="detached" title="" "#,
                r#"href="/node/SomeOtherAnchor">SomeOtherAnchor</a></p>"#,
            ),
        );
    }

    #[test]
    fn trailing_anchor() {
        assert_eq!(
            read("see acks|acks"),
            concat!(
                r#"<p>see <a class="detached" title="" "#,
                r#"href="/node/acks">acks</a></p>"#,
            )
        );
    }

    #[test]
    fn trailing_anchor_with_newline() {
        assert_eq!(
            read("\nsee acks|acks\n"),
            concat!(
                "\n",
                r#"<p>see <a class="detached" title="" "#,
                r#"href="/node/acks">acks</a></p>"#,
            ),
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
            concat!(
                r#"<p>the <a class="detached" title="" "#,
                r#"href="/node/lion">lion</a>'s mouth</p>"#,
            )
        );
    }

    #[test]
    fn anchor_with_trailing_double_quote() {
        assert_eq!(
            read(r#"the "|real|" motive"#),
            concat!(
                r#"<p>the "<a class="detached" title="" "#,
                r#"href="/node/real">real</a>" motive</p>"#,
            )
        );
    }

    #[test]
    fn anchor_with_trailing_parenthesis() {
        assert_eq!(
            read("this (though |true|) was questioned"),
            concat!(
                r#"<p>this (though <a class="detached" title="" "#,
                r#"href="/node/true">true</a>) was questioned</p>"#,
            )
        );
    }

    #[test]
    fn anchor_with_leading_single_quote() {
        assert_eq!(
            read("the 'real|Reality' motive"),
            concat!(
                r#"<p>the '<a class="detached" title="" "#,
                r#"href="/node/Reality">real</a>' motive</p>"#,
            )
        );
    }

    #[test]
    fn anchor_with_leading_double_quote() {
        assert_eq!(
            read(r#"the "real|Reality" motive"#),
            concat!(
                r#"<p>the "<a class="detached" title="" "#,
                r#"href="/node/Reality">real</a>" motive</p>"#,
            )
        );
    }

    #[test]
    fn anchor_with_leading_parenthesis() {
        assert_eq!(
            read("her (last|Surname) name"),
            concat!(
                r#"<p>her (<a class="detached" title="" "#,
                r#"href="/node/Surname">last</a>) name</p>"#,
            )
        );
    }

    #[test]
    fn anchor_with_internal_apostrophe() {
        assert_eq!(
            read("the |lion's mouth|album was released"),
            concat!(
                r#"<p>the <a class="detached" title="" "#,
                r#"href="/node/album">lion's mouth</a> was released</p>"#,
            )
        );
    }

    #[test]
    fn nonleading_anchor_with_internal_apostrophe() {
        assert_eq!(
            read("they decided to stay at Jane's|YellowHouse that night"),
            concat!(
                r#"<p>they decided to stay at <a class="detached" title="" "#,
                r#"href="/node/YellowHouse">Jane's</a> that night</p>"#,
            )
        );
    }

    #[test]
    fn nonleading_anchor_with_internal_apostrophe_at_eoi() {
        assert_eq!(
            read("they decided to stay at Jane's|YellowHouse"),
            concat!(
                r#"<p>they decided to stay at <a class="detached" "#,
                r#"title="" href="/node/YellowHouse">Jane's</a></p>"#,
            )
        );
    }

    #[test]
    fn nonleading_anchor_with_internal_apostrophe_at_soi() {
        assert_eq!(
            read("Jane's|YellowHouse that night"),
            concat!(
                r#"<p><a class="detached" title="" "#,
                r#"href="/node/YellowHouse">Jane's</a> that night</p>"#,
            )
        );
    }

    #[test]
    fn anchor_with_internal_double_quotes() {
        assert_eq!(
            read(r#"the |"real"|Truth motive"#),
            concat!(
                r#"<p>the <a class="detached" title="" "#,
                r#"href="/node/Truth">"real"</a> motive</p>"#,
            )
        );
    }

    #[test]
    fn anchor_with_internal_double_quotes_wrapping_spaced_words() {
        assert_eq!(
            read(r#"the |"bare reality"|Ideology they believed"#),
            concat!(
                r#"<p>the <a class="detached" title="" "#,
                r#"href="/node/Ideology">"bare reality"</a> they believed</p>"#,
            )
        );
    }

    #[test]
    fn anchor_with_internal_parenthesis() {
        assert_eq!(
            read("her |last (name)|Surname was Amad"),
            concat!(
                r#"<p>her <a class="detached" title="" "#,
                r#"href="/node/Surname">last (name)</a> was Amad</p>"#,
            )
        );
    }

    #[test]
    fn anchor_with_internal_parenthesis_wrapping_spaced_words() {
        assert_eq!(
            read("this |truth (though questionable) was fine|Absurd to them "),
            concat!(
                r#"<p>this <a class="detached" title="" "#,
                r#"href="/node/Absurd">truth (though questionable) was "#,
                r#"fine</a> to them</p>"#,
            )
        );
    }
}
