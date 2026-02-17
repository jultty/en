use std::{iter::Peekable, slice::Iter};

use crate::{
    graph::Graph,
    prelude::*,
    syntax::content::parser::{
        Lexeme, State, Token, context::Block, format, state, token::Table,
    },
};

/// Handles open table contexts until a table is fully parsed.
///
/// A return of `true` will trigger a continue in the outer parser,
/// skipping any further parsing of the current lexeme.
///
/// # Panics
/// This parser can handle only the Table context, and will panic if passed an
/// unrelated context since it has no knowledge on how to handle them.
pub fn parse(
    lexeme: &Lexeme,
    state: &mut State,
    tokens: &mut Vec<Token>,
    iterator: &mut Peekable<Iter<'_, Lexeme>>,
    graph: &Graph,
) -> bool {
    let buffer = &mut state.buffers.table;
    let candidate = &mut buffer.candidate;

    let mut parse_text = |text: &str| {
        let (parsed_text, text_tokens) = format(text, graph);
        state.format_tokens.extend_from_slice(&text_tokens);
        parsed_text
    };

    #[expect(clippy::wildcard_enum_match_arm)]
    match state.context.block {
        Block::Table => {
            if Table::probe_end(lexeme) {
                log!(VERBOSE, "Probed end of table on {lexeme}");

                if buffer.in_header {
                    log!(VERBOSE, "Adding unterminated header: {lexeme}");
                    candidate.add_header(&parse_text(&buffer.cell));
                } else {
                    let descriptor = if buffer.in_cell {
                        "unterminated"
                    } else {
                        "undelimited"
                    };
                    log!(VERBOSE, "Adding {descriptor} cell: {lexeme}");
                    candidate.add_cell(&parse_text(&buffer.cell));
                }

                tokens.push(Token::Table(candidate.clone()));
                log!(VERBOSE, "Block Context: Table -> None on {lexeme}");
                state.context.block = Block::None;
                *buffer = state::TableBuffer::default();
                iterator.next();
            } else if lexeme.match_char('\n')
                || lexeme.match_char_triple(' ', '!', '\n')
                || lexeme.match_char_triple(' ', '|', '\n')
            {
                log!(VERBOSE, "Adding row: found newline on {lexeme}");

                if !buffer.cell.is_empty() {
                    if buffer.in_header {
                        log!(VERBOSE, "Adding unterminated header: {lexeme}");
                        candidate.add_header(&parse_text(&buffer.cell));
                    } else {
                        let descriptor = if buffer.in_cell {
                            "unterminated"
                        } else {
                            "undelimited"
                        };
                        log!(VERBOSE, "Adding {descriptor} cell: {lexeme}");
                        candidate.add_cell(&parse_text(&buffer.cell));
                    }
                    buffer.cell.clear();
                }

                if lexeme.match_next_either_char('|', '!') {
                    iterator.next();
                }

                buffer.in_header = false;
                buffer.in_cell = false;
                candidate.add_row(vec![]);
            } else if lexeme.match_char_triple(' ', '!', ' ') {
                buffer.in_header = true;
                if !buffer.cell.trim().is_empty() {
                    log!(VERBOSE, "Adding header: found spaced ! on {lexeme}");
                    candidate.add_header(&parse_text(&buffer.cell));
                    buffer.cell.clear();
                }
                iterator.next();
                iterator.next();
            } else if lexeme.match_char_triple(' ', '|', ' ') {
                buffer.in_cell = true;
                if !buffer.cell.trim().is_empty() {
                    log!(VERBOSE, "Adding cell: found spaced | on {lexeme}");
                    candidate.add_cell(&parse_text(&buffer.cell));
                    buffer.cell.clear();
                }
                iterator.next();
                iterator.next();
            } else {
                log!(VERBOSE, "Extending cell text on {lexeme}");
                buffer.cell.push_str(lexeme.text().as_str());
            }
        },
        _ => {
            panic!("Table context parser called to handle non-table context")
        },
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::{graph::Graph, syntax::content::parser};

    fn read(input: &str) -> String { parser::read(input, &Graph::default()) }

    fn read_loaded(input: &str) -> String {
        parser::read(input, &Graph::load())
    }

    #[test]
    fn single_row() {
        assert_eq!(
            read(concat!("%", "\n", "a | b | c", "\n", "%", "\n")),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <td>a</td>", "\n",
                "        <td>b</td>", "\n",
                "        <td>c</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }

    #[test]
    fn two_rows() {
        assert_eq!(
            read(concat!(
                "%", "\n",
                "a | b | c", "\n",
                "d | e | f", "\n",
                "%", "\n",
            )),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <td>a</td>", "\n",
                "        <td>b</td>", "\n",
                "        <td>c</td>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>d</td>", "\n",
                "        <td>e</td>", "\n",
                "        <td>f</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }

    #[test]
    fn three_rows() {
        assert_eq!(
            read(concat!(
                "%", "\n",
                "a | b | c", "\n",
                "d | e | f", "\n",
                "g | h | i", "\n",
                "%", "\n",
            )),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <td>a</td>", "\n",
                "        <td>b</td>", "\n",
                "        <td>c</td>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>d</td>", "\n",
                "        <td>e</td>", "\n",
                "        <td>f</td>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>g</td>", "\n",
                "        <td>h</td>", "\n",
                "        <td>i</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }

    #[test]
    fn with_header() {
        assert_eq!(
            read(concat!(
                "%", "\n",
                "hA ! hB ! hC", "\n",
                "a | b | c", "\n",
                "d | e | f", "\n",
                "%", "\n",
            )),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <th>hA</th>", "\n",
                "        <th>hB</th>", "\n",
                "        <th>hC</th>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>a</td>", "\n",
                "        <td>b</td>", "\n",
                "        <td>c</td>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>d</td>", "\n",
                "        <td>e</td>", "\n",
                "        <td>f</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }

    #[test]
    fn with_anchor() {
        assert_eq!(
            read(concat!(
                "%", "\n",
                "a | |Node| | c", "\n",
                "%", "\n",
            )),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <td>a</td>", "\n",
                r#"        <td><a class="detached" title="" "#,
                r#"href="/node/Node">Node</a></td>"#, "\n",
                "        <td>c</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }

    #[test]
    fn with_loaded_anchor() {
        assert_eq!(
            read_loaded(concat!(
                "%", "\n",
                "a | |Node| | c", "\n",
                "%", "\n",
            )),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <td>a</td>", "\n",
                r#"        <td><a class="attached" title="A node is defined "#,
                r#"in your graph file starting with a table header of the "#,
                r#"form:" href="/node/Node">Node</a></td>"#, "\n",
                "        <td>c</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }

    #[test]
    fn no_leading_delimiters() {
        assert_eq!(
            read_loaded(concat!(
                "%", "\n",
                "a ! b ! c !", "\n",
                "d | e | f |", "\n",
                "g | h | i |", "\n",
                "%", "\n",
            )),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <th>a</th>", "\n",
                "        <th>b</th>", "\n",
                "        <th>c</th>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>d</td>", "\n",
                "        <td>e</td>", "\n",
                "        <td>f</td>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>g</td>", "\n",
                "        <td>h</td>", "\n",
                "        <td>i</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }

    #[test]
    fn no_trailing_delimiters() {
        assert_eq!(
            read_loaded(concat!(
                "%", "\n",
                " ! a ! b ! c", "\n",
                " | d | e | f", "\n",
                " | g | h | i", "\n",
                "%", "\n",
            )),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <th>a</th>", "\n",
                "        <th>b</th>", "\n",
                "        <th>c</th>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>d</td>", "\n",
                "        <td>e</td>", "\n",
                "        <td>f</td>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>g</td>", "\n",
                "        <td>h</td>", "\n",
                "        <td>i</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }

    #[test]
    fn with_leading_and_trailing_delimiters() {
        assert_eq!(
            read_loaded(concat!(
                "%", "\n",
                " ! a ! b ! c !", "\n",
                " | d | e | f |", "\n",
                " | g | h | i |", "\n",
                "%", "\n",
            )),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <th>a</th>", "\n",
                "        <th>b</th>", "\n",
                "        <th>c</th>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>d</td>", "\n",
                "        <td>e</td>", "\n",
                "        <td>f</td>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>g</td>", "\n",
                "        <td>h</td>", "\n",
                "        <td>i</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }

    #[test]
    fn no_flanking_delimiters() {
        assert_eq!(
            read_loaded(concat!(
                "%", "\n",
                "a ! b ! c", "\n",
                "d | e | f", "\n",
                "g | h | i", "\n",
                "%", "\n",
            )),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <th>a</th>", "\n",
                "        <th>b</th>", "\n",
                "        <th>c</th>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>d</td>", "\n",
                "        <td>e</td>", "\n",
                "        <td>f</td>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>g</td>", "\n",
                "        <td>h</td>", "\n",
                "        <td>i</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }

    #[test]
    fn with_indent() {
        assert_eq!(
            read_loaded(concat!(
                "%", "\n",
                "    ! a ! b ! c !", "\n",
                "           | d | e | f |", "\n",
                " | g | h | i |", "\n",
                "%", "\n",
            )),
            concat!(
                "\n",
                "<table>", "\n",
                "    <tr>", "\n",
                "        <th>a</th>", "\n",
                "        <th>b</th>", "\n",
                "        <th>c</th>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>d</td>", "\n",
                "        <td>e</td>", "\n",
                "        <td>f</td>", "\n",
                "    </tr>", "\n",
                "    <tr>", "\n",
                "        <td>g</td>", "\n",
                "        <td>h</td>", "\n",
                "        <td>i</td>", "\n",
                "    </tr>", "\n",
                "</table>", "\n",
        )
        );
    }
}
