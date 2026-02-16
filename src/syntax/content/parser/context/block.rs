use std::{iter::Peekable, slice::Iter};

use crate::{
    graph::Graph,
    prelude::*,
    syntax::content::{
        Parseable as _,
        parser::{
            Block, Lexeme, State, Token,
            token::{
                Header, List, LineBreak, Literal, Paragraph, PreFormat, Quote,
                Table, Verse,
            },
        },
    },
};

pub fn parse(
    lexeme: &Lexeme,
    state: &mut State,
    tokens: &mut Vec<Token>,
    iterator: &mut Peekable<Iter<'_, Lexeme>>,
    graph: &Graph,
) -> bool {
    match state.context.block {
        Block::None => {
            if PreFormat::probe(lexeme) {
                log!(VERBOSE, "Block Context: None -> PreFormat on {lexeme}");
                state.context.block = Block::PreFormat;
                tokens.push(Token::PreFormat(PreFormat::new(true)));
                return true;
            } else if Header::probe(lexeme) {
                let mut header = Header::lex(lexeme);
                header.dom_id = Some(Header::make_id(
                    &graph.meta.config,
                    iterator.peek().map_or(&Lexeme::default(), |l| l),
                    &mut state.dom_ids,
                ));
                log!(VERBOSE, "Block Context: None -> Header on {lexeme}");
                state.context.block = Block::Header(header.level());
                tokens.push(Token::Header(header));
                return true;
            } else if List::probe(lexeme) {
                log!(VERBOSE, "Block Context: None -> List on {lexeme}");
                state.context.block = Block::List;
                state.buffers.list.candidate.ordered = lexeme.match_char('+');
                return super::list::parse(
                    lexeme, state, tokens, iterator, graph,
                );
            } else if Quote::probe(lexeme) {
                log!(VERBOSE, "Block Context: None -> Quote on {lexeme}");
                state.context.block = Block::Quote;
                iterator.next();
                return true;
            } else if Verse::probe(lexeme) {
                log!(VERBOSE, "Block Context: None -> Verse on {lexeme}");
                state.context.block = Block::Verse;
                tokens.push(Token::Verse(Verse::new(true)));
                iterator.next();
                iterator.next();
                return true;
            } else if Table::probe(lexeme) {
                log!(VERBOSE, "Block Context: None -> Table on {lexeme}");
                state.context.block = Block::Table;
                iterator.next();
                return true;
            } else if Paragraph::probe(lexeme) {
                log!(VERBOSE, "Block Context: None -> Paragraph on {lexeme}");
                state.context.block = Block::Paragraph;
                tokens.push(Token::Paragraph(Paragraph::new(true)));
            }
        },
        Block::PreFormat => {
            if PreFormat::probe(lexeme) {
                tokens.push(Token::PreFormat(PreFormat::new(false)));
                log!(VERBOSE, "Block Context: PreFormat -> None on {lexeme}");
                state.context.block = Block::None;
            } else {
                tokens.push(Token::Literal(Literal::lex(lexeme)));
            }
            return true;
        },
        Block::Paragraph => {
            if Paragraph::probe_end(lexeme) {
                tokens.push(Token::Paragraph(Paragraph::new(false)));
                log!(VERBOSE, "Block Context: Paragraph -> None on {lexeme}");
                state.context.block = Block::None;
            }
        },
        Block::Header(n) => {
            if lexeme.text() == "\n" {
                tokens.push(Token::Header(Header::from_u8(n, false, None)));
                log!(VERBOSE, "Block Context: Header -> None on {lexeme}");
                state.context.block = Block::None;
            }
        },
        Block::List => {
            return super::list::parse(lexeme, state, tokens, iterator, graph);
        },
        Block::Quote => {
            return super::quote::parse(lexeme, state, tokens, iterator, graph);
        },
        Block::Table => {
            return super::table::parse(lexeme, state, tokens, iterator, graph);
        },
        Block::Verse => {
            if Verse::probe_end(lexeme) {
                tokens.push(Token::Verse(Verse::new(false)));
                log!(VERBOSE, "Block Context: Verse -> None on {lexeme}");
                state.context.block = Block::None;
                iterator.next();
                iterator.next();
                return true;
            } else if lexeme.match_char('\n') {
                tokens.push(Token::LineBreak(LineBreak::default()));
                return true;
            }
        },
    }
    false
}

#[cfg(test)]
mod tests {

    use crate::{
        syntax::content::parser::{
            self, Block, Token, context, State,
            token::{Header, header::Level, PreFormat},
        },
        graph::Graph,
    };

    fn read(input: &str) -> String {
        parser::read(input, &Graph::default())
    }

    #[test]
    fn pre() {
        let payload = "D0qdJ184f3q1okbYu3Xm1d93jj6jy615";
        assert_eq!(
            read(&format!("`\n{payload}\n`\n")),
            format!("<pre>\n{payload}\n</pre>"),
        );
    }

    #[test]
    fn eoi_pre() {
        let payload = "Jp8INpWzsQmk20jpIhBFCfMUXOztxv0w";
        assert_eq!(
            read(&format!("`\n{payload}\n`")),
            format!("<pre>\n{payload}\n</pre>"),
        );
    }

    #[test]
    fn end_with_open_header() {
        let mut state = State::default();
        state.context.block = Block::Header(1);

        let mut vec: Vec<Token> = vec![];
        context::close(&state, &mut vec);
        assert_eq!(vec, vec![Token::Header(Header::from_u8(1, false, None))]);
    }

    #[test]
    fn end_with_open_preformat() {
        let mut state = State::default();
        state.context.block = Block::PreFormat;

        let mut vec: Vec<Token> = vec![];
        context::close(&state, &mut vec);
        assert_eq!(vec, vec![Token::PreFormat(PreFormat::new(false))]);
    }

    #[test]
    fn truncated_header_level() {
        let u: usize = 999;
        let level = Level::from(u);
        assert_eq!(level.to_string(), "6");
    }
}
