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

    #[allow(clippy::wildcard_enum_match_arm)]
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
