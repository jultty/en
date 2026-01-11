use std::{iter::Peekable, slice::Iter};

use crate::{
    prelude::*,
    syntax::content::parser::{
        context::Block,
        lexeme::Lexeme,
        nest,
        state::{ListBuffer, State},
        token::{Token, item::Item},
    },
    types::Graph,
};

/// Handles open list contexts until a list is fully parsed.
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
    graph: &Graph,
) -> bool {
    let buffer = &mut state.buffers.list;
    let candidate = &mut buffer.candidate;
    let item_candidate = &mut buffer.item_candidate;

    match state.context.block {
        Block::List => {
            if lexeme.match_char(' ') && item_candidate.depth.is_none() {
                // found space, unknown increasing depth
                buffer.depth = buffer.depth.saturating_add(1);
            } else if item_candidate.depth.is_none()
                && lexeme.match_either_char('-', '+')
            {
                // found bullet, depth now known
                item_candidate.depth = Some(buffer.depth);
                if lexeme.match_next_char(' ') {
                    iterator.next();
                }
            } else if lexeme.last() || lexeme.match_char_sequence('\n', '\n') {
                // found end of list
                if !lexeme.match_char('\n') {
                    // no trailing break, last item's text wouldn't be pushed
                    item_candidate.text.push_str(&lexeme.text());
                }
                if item_candidate.depth.is_some() {
                    // if the current item candidate has a known depth, push it
                    item_candidate.text = nest(&item_candidate.text, graph);
                    candidate.items.push(item_candidate.clone());
                }
                // push list candidate, reset state and exit context
                log!("Accepting list candidate {candidate}");
                tokens.push(Token::List(candidate.clone()));
                state.context.block = Block::None;
                iterator.next();
                *buffer = ListBuffer::default();
            } else if lexeme.match_char('\n') {
                // found end of item, push it and reset state
                log!("Accepting item candidate {item_candidate}");
                item_candidate.text = nest(&item_candidate.text, graph);
                candidate.items.push(item_candidate.clone());
                *item_candidate = Item::default();
                buffer.depth = 0;
            } else {
                // anything else is pushed into the candidate item's text
                item_candidate.text.push_str(&lexeme.text());
            }
        },
        Block::None
        | Block::Paragraph
        | Block::Header(_)
        | Block::PreFormat => {
            panic!("List context parser called to handle non-list context")
        },
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::{
        syntax::content::parser::{
            self, context::list::parse, lexeme::Lexeme, state::State,
        },
        types::Graph,
    };

    fn read(input: &str) -> String {
        parser::read(input, &Graph::default())
    }

    #[test]
    fn unordered_list() {
        assert_eq!(
            read("- a\n- b\n- c"),
            "\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            <li>c</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn minimally_nested_unordered_list() {
        assert_eq!(
            read("- a\n  - b"),
            "\n<ul>\n\
            <li>a<ul>\n\
            <li>b</li></ul></li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn unordered_nested_list() {
        assert_eq!(
            read(concat!(
                "- 0Aa\n",
                "    - 4Ba\n",
                "- 0Ca\n",
                "    - 4Da\n",
                "    - 4Db\n",
                "- 0Ea\n",
                "- 0Eb"
            )),
            "\n<ul>\n\
                <li>0Aa<ul>\n\
                <li>4Ba</li></ul></li>\n\
                <li>0Ca<ul>\n\
                <li>4Da</li>\n\
                <li>4Db</li></ul></li>\n\
                <li>0Ea</li>\n\
                <li>0Eb</li>\n\
                </ul>\n\n"
        );
    }

    #[test]
    fn unordered_list_as_eoi() {
        assert_eq!(
            read("some\n\n- a\n- b"),
            "<p>some</p>\n\n\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn unordered_list_as_soi() {
        assert_eq!(
            read("- a\n- b\n\nsome"),
            "\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            </ul>\n\n<p>some</p>"
        );
    }

    #[test]
    fn unordered_list_after_newline() {
        assert_eq!(
            read("\n- a\n- b"),
            "\n\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn unordered_list_after_two_newlines() {
        assert_eq!(
            read("\n\n- a\n- b"),
            "\n\n\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn unordered_list_after_three_newlines() {
        assert_eq!(
            read("\n\n\n- a\n- b"),
            "\n\n\n\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn unordered_list_before_newline() {
        assert_eq!(
            read("- a\n- b\n"),
            "\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn unordered_list_before_two_newlines() {
        assert_eq!(
            read("- a\n- b\n\n"),
            "\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn unordered_list_before_three_newlines() {
        assert_eq!(
            read("- a\n- b\n\n\n"),
            "\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    fn unordered_list_before_three_newlines_then_literals() {
        assert_eq!(
            read("- a\n- b\n\n\nw w"),
            "\n<ul>\n\
            <li>a</li>\n\
            <li>b</li>\n\
            </ul>\n\n\n<p>w w</p>"
        );
    }

    #[test]
    fn unordered_nested_list_multilevel_depth_drop() {
        assert_eq!(
            read(concat!(
                "- 0Aa\n",
                "    - 4Ba\n",
                "        - 8Ca\n",
                "            - 12Da\n",
                "                - 16Ea\n",
                "        - 8Fa\n",
                "- 0Ga\n\n",
            )),
            "\n<ul>\n\
            <li>0Aa<ul>\n\
            <li>4Ba<ul>\n\
            <li>8Ca<ul>\n\
            <li>12Da<ul>\n\
            <li>16Ea</li></ul></li></ul></li>\n\
            <li>8Fa</li></ul></li></ul></li>\n\
            <li>0Ga</li>\n\
            </ul>\n\n"
        );
    }

    #[test]
    #[should_panic(
        expected = "List context parser called to handle non-list context"
    )]
    fn bad_context() {
        let mut state = State::default();
        let lexemes = Lexeme::collect(&["a", "b", "c"].map(str::to_string));
        let graph = Graph::default();
        parse(
            &Lexeme::default(),
            &mut state,
            &mut vec![],
            &mut lexemes.iter().peekable(),
            &graph,
        );
    }
}
