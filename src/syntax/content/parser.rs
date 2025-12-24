use std::collections::{HashMap};

use crate::{formats::populate_graph, types::Config};
use super::{Parseable as _, Token, LexMap};
use token::{
    anchor::Anchor, linebreak::LineBreak, paragraph::Paragraph, header::Header,
    preformat::PreFormat, literal::Literal, code::Code,
};
use lexeme::Lexeme;

pub mod token;
pub mod lexeme;
pub mod segment;

const LEXMAP: LexMap = &[
    (LineBreak::probe, |word| {
        Token::LineBreak(LineBreak::lex(word))
    }),
    (Literal::probe, |word| Token::Literal(Literal::lex(word))),
];

fn lex(text: &str, map: LexMap) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut state = State::new();
    let config: Config = populate_graph().meta.config;

    let segments = segment::segment(text);
    let lexemes = Lexeme::collect(&segments);

    let mut iterator = lexemes.iter().peekable();
    while let Some(lexeme) = iterator.next() {
        match state.context.block {
            BlockContext::None => {
                if PreFormat::probe(lexeme) {
                    state.context.block = BlockContext::PreFormat;
                    tokens.push(Token::PreFormat(PreFormat::new(true)));
                    continue;
                } else if Header::probe(lexeme) {
                    let mut header = Header::lex(lexeme);
                    header.dom_id = Some(Header::make_id(
                        &config,
                        &mut iterator,
                        &mut state.dom_ids,
                    ));
                    state.context.block = BlockContext::Header(header.level());
                    tokens.push(Token::Header(header));
                    continue;
                } else if Paragraph::probe(lexeme) {
                    state.context.block = BlockContext::Paragraph;
                    tokens.push(Token::Paragraph(Paragraph::new(true)));
                }
            },
            BlockContext::PreFormat => {
                if PreFormat::probe(lexeme) {
                    tokens.push(Token::PreFormat(PreFormat::new(false)));
                    state.context.block = BlockContext::None;
                } else {
                    tokens.push(Token::Literal(Literal::lex(lexeme)));
                }
                continue;
            },
            BlockContext::Paragraph => {
                if lexeme.text() == "\n" {
                    tokens.push(Token::Paragraph(Paragraph::new(false)));
                    state.context.block = BlockContext::None;
                }
            },
            BlockContext::Header(n) => {
                if lexeme.text() == "\n" {
                    tokens.push(Token::Header(Header::from_u8(n, false, None)));
                    state.context.block = BlockContext::None;
                }
            },
        }

        match state.context.inline {
            InlineContext::None => {
                if Code::probe(lexeme) {
                    state.context.inline = InlineContext::Code;
                    tokens.push(Token::Code(Code::new(true)));
                    continue;
                } else if Anchor::probe(lexeme) {
                    state.context.inline = InlineContext::Anchor;
                    state.buffers.anchor.clear();

                    if lexeme.match_first_char('|') {
                        state.buffers.anchor.candidate.leading = true;
                    } else {
                        state.buffers.anchor.candidate.text = lexeme.text();
                    }
                    continue;
                }
            },
            InlineContext::Code => {
                if Code::probe(lexeme) {
                    state.context.inline = InlineContext::None;
                    tokens.push(Token::Code(Code::new(false)));
                    continue;
                }
            },
            InlineContext::Anchor => {
                let buffer = &mut state.buffers.anchor;
                let candidate = &mut buffer.candidate;
                if candidate.text.is_empty() {
                    if lexeme.next == "|" {
                        buffer.text.push_str(&lexeme.text());
                        candidate.text.clone_from(&buffer.text);
                    } else {
                        buffer.text.push_str(&lexeme.text());
                    }
                    continue;
                } else if candidate.destination.is_none() {
                    // candidate is leading and we found the second pipe
                    if candidate.leading && lexeme.text() == "|" {
                        // third pipe immediately after second: forcing flanking
                        if lexeme.match_next_first_char('|') {
                            continue;
                        // whitespace or punctuation after pipe: flanking anchor
                        } else if lexeme.is_next_whitespace()
                            || lexeme.is_next_punctuation()
                        {
                            candidate.destination =
                                Some(candidate.text.clone());
                            let token = Token::Anchor(candidate.clone());
                            tokens.push(token);
                            state.context.inline = InlineContext::None;
                        // non-whitespace after pipe is the destination
                        } else {
                            candidate.destination = Some(lexeme.next.clone());
                            let token = Token::Anchor(candidate.clone());
                            tokens.push(token);
                            state.context.inline = InlineContext::None;
                            // if there is a trailing pipe, consume it
                            if let Some(next) = iterator.next()
                                && next.next == "|"
                            {
                                iterator.next();
                            }
                        }
                    // candidate is nonleading and we found a second pipe
                    } else if !candidate.leading && lexeme.next == "|" {
                        candidate.destination = Some(lexeme.text());
                        tokens.push(Token::Anchor(candidate.clone()));
                        state.context.inline = InlineContext::None;
                        iterator.next();
                    // candidate is nonleading and we found whitespace
                    } else if lexeme.is_next_whitespace() {
                        candidate.destination = Some(lexeme.text());
                        let token = Token::Anchor(candidate.clone());
                        tokens.push(token);
                        state.context.inline = InlineContext::None;
                    // candidate is nonleading and we haven't found whitespace
                    } else {
                        buffer.destination.push_str(&lexeme.text());
                    }
                    continue;
                } else {
                    unreachable!("Anchor is already fully parsed");
                }
            },
        }

        for &(ref probe, lex) in map {
            if probe(lexeme) {
                tokens.push(lex(lexeme));
                break;
            }
        }
    }

    tokens
}

enum BlockContext {
    Paragraph,
    Header(u8),
    PreFormat,
    None,
}

enum InlineContext {
    Anchor,
    Code,
    None,
}

struct State {
    context: Context,
    dom_ids: HashMap<String, Vec<String>>,
    buffers: Buffers,
}

struct Context {
    block: BlockContext,
    inline: InlineContext,
}

struct Buffers {
    anchor: AnchorBuffer,
}

#[derive(Debug)]
struct AnchorBuffer {
    candidate: Anchor,
    text: String,
    destination: String,
}

impl AnchorBuffer {
    fn clear(&mut self) {
        self.candidate = Anchor::empty();
        self.text = String::new();
        self.destination = String::new();
    }
}

impl State {
    fn new() -> State {
        State {
            context: Context {
                inline: InlineContext::None,
                block: BlockContext::None,
            },
            dom_ids: HashMap::new(),
            buffers: Buffers {
                anchor: AnchorBuffer {
                    candidate: Anchor::empty(),
                    text: String::new(),
                    destination: String::new(),
                },
            },
        }
    }
}

fn parse(tokens: &[Token]) -> String {
    tokens.iter().map(Token::render).collect::<String>()
}

pub(super) fn read(text: &str) -> String {
    parse(&lex(text, LEXMAP))
}
