use std::collections::{HashMap, hash_map::Entry};

use crate::{formats::populate_graph, types::Config};

use super::{Parseable as _, Token, LexMap};
use token::{
    anchor::Anchor, linebreak::LineBreak, paragraph::Paragraph, header::Header,
    preformat::PreFormat, literal::Literal, code::Code,
};
use lexeme::Lexeme;

pub mod token;
pub mod lexeme;
pub mod cluster;

const LEXMAP: LexMap = &[
    (LineBreak::probe, |word| {
        Token::LineBreak(LineBreak::lex(word))
    }),
    (Code::probe, |word| Token::Code(Code::lex(word))),
    (Anchor::probe, |word| Token::Anchor(Anchor::lex(word))),
    (Literal::probe, |word| Token::Literal(Literal::lex(word))),
];

enum Context {
    None,
    Paragraph,
    Header(u8),
    PreFormat,
}

struct State {
    context: Context,
    dom_ids: HashMap<String, Vec<String>>,
}

fn lex(text: &str, map: LexMap) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut state = State {
        context: Context::None,
        dom_ids: HashMap::new(),
    };
    let config: Config = populate_graph().meta.config;

    let splits = cluster::cluster(text);
    let lexemes = Lexeme::collect(&splits);
    let iter = lexemes.iter().peekable();
    for lexeme in iter {
        match state.context {
            Context::None => {
                if PreFormat::probe(lexeme) {
                    tokens.push(Token::PreFormat(PreFormat::new(true)));
                    state.context = Context::PreFormat;
                    continue;
                } else if Header::probe(lexeme) {
                    let base_id =
                        if config.ascii_dom_ids && !lexeme.next.is_ascii() {
                            String::from("h")
                        } else {
                            lexeme.next.clone().to_lowercase()
                        };
                    let id = match state.dom_ids.entry(base_id.clone()) {
                        Entry::Occupied(mut occupied) => {
                            let ids = occupied.get_mut();
                            let suffix: u8 =
                                ids.len().try_into().unwrap_or_default();
                            let id_with_suffix = format!("{base_id}-{suffix}");
                            ids.push(id_with_suffix.clone());
                            id_with_suffix
                        },
                        Entry::Vacant(vacant) => {
                            vacant.insert(vec![base_id.clone()]);
                            base_id
                        },
                    };

                    let mut header = Header::lex(lexeme);
                    header.dom_id = Some(id);
                    state.context = Context::Header(header.get_level());
                    tokens.push(Token::Header(header));
                    continue;
                } else if Paragraph::probe(lexeme) {
                    tokens.push(Token::Paragraph(Paragraph::new(true)));
                    state.context = Context::Paragraph;
                }
            },
            Context::PreFormat => {
                if PreFormat::probe(lexeme) {
                    tokens.push(Token::PreFormat(PreFormat::new(false)));
                    state.context = Context::None;
                } else {
                    tokens.push(Token::Literal(Literal::lex(lexeme)));
                }
                continue;
            },
            Context::Paragraph => {
                if lexeme.text() == "\n" {
                    tokens.push(Token::Paragraph(Paragraph::new(false)));
                    state.context = Context::None;
                }
            },
            Context::Header(n) => {
                if lexeme.text() == "\n" {
                    tokens.push(Token::Header(Header::from_u8(n, false, None)));
                    state.context = Context::None;
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

fn parse(tokens: &[Token]) -> String {
    tokens.iter().map(Token::render).collect::<String>()
}

pub(super) fn read(text: &str) -> String {
    parse(&lex(text, LEXMAP))
}
